use anyhow::{Error, Result};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use chacha20poly1305::{
    aead::{Aead, NewAead, Payload},
    ChaCha20Poly1305, Key, Nonce,
};

use crate::{
    net::{address::Address, socket::Socket},
    protos::{Protocol, ProtocolType, ResolvedResult},
    utils,
    utils::crypto::Crypto,
    ServiceType,
};

const MAX_CHUNK_SIZE: usize = 0x3FFF;
const SALT_SIZE: usize = 32;
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const HKDF_INFO: &str = "bp-subkey";

/// # Protocol
///
/// Header
/// +--------+------------+-----+
/// |  Salt  | DataFrame  | ... |
/// +--------+------------+-----+
/// |   32   |  Variable  | ... |
/// +--------+------------+-----+
///
/// DataFrame
/// +------------+----------------+------------+-----------+---------------+------------+-----------+
/// | PaddingLen | PaddingLen Tag |  Padding   |  ChunkLen |  ChunkLen Tag |   Chunk    | Chunk Tag |
/// +------------+----------------+------------+-----------+---------------+------------+-----------+
/// |     1      |       16       |  Variable  |     2     |       16      |  Variable  |    16     |
/// +------------+----------------+------------+-----------+---------------+------------+-----------+
///
/// First Chunk
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
///
/// # Explain
///
/// * Salt is randomly generated, and is to derive the per-session subkey in HKDF.
/// * AEAD cipher ChaCha20Poly1305 (RFC 8439) is used to encrypt all DataFrames.
/// * Nonce is little-endian and counting from 0, each chunk increases the nonce three times.
/// * The HMAC-based Extract-and-Expand Key Derivation Function(HKDF) is used for key derivation.
/// * The HKDF use SHA256 hash function.
/// * The random salt and info = "bp-subkey" is used to HKDF.
/// * The length of Chunk Data must <= 0x3FFF.
/// * Only PaddingLen, ChunkLen, Chunk are encrypted.
///
/// # Reference
///
/// * chacha20poly1305: https://docs.rs/chacha20poly1305/0.8.1/chacha20poly1305/
/// * HKDF: https://docs.rs/hkdf/0.11.0/hkdf/
#[derive(Clone)]
pub struct Erp {
    header_sent: bool,

    raw_key: String,

    salt: Option<Bytes>,

    derived_key: Option<Bytes>,

    encrypt_nonce: u128,

    decrypt_nonce: u128,

    resolved_result: Option<ResolvedResult>,
}

impl Erp {
    pub fn new(key: String, service_type: ServiceType) -> Self {
        let (salt, derived_key) = match service_type {
            ServiceType::Server => (None, None),
            // only client side can generate salt and derived_key
            // generate on server side will take no effect.
            ServiceType::Client => {
                let salt = Crypto::random_bytes(SALT_SIZE);
                let derived_key = Self::derive_key(key.clone(), salt.clone());
                (Some(salt), Some(derived_key))
            }
        };

        Self {
            raw_key: key,
            salt,
            derived_key,
            encrypt_nonce: 0,
            decrypt_nonce: 0,
            header_sent: false,
            resolved_result: None,
        }
    }

    fn derive_key(raw_key: String, salt: Bytes) -> Bytes {
        let derived_key = Crypto::hkdf_sha256(raw_key.into(), salt, HKDF_INFO.as_bytes().into(), KEY_SIZE);
        log::debug!("encrypt/decrypt key = {}", utils::fmt::ToHex(derived_key.to_vec()));
        derived_key
    }

    fn encrypt(&mut self, plain_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.encrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        log::debug!("encrypt nonce = {}", utils::fmt::ToHex(nonce.to_vec()));
        log::debug!("encrypt plain_text = {}", utils::fmt::ToHex(plain_text.to_vec()));

        let plain_text = Payload::from(&plain_text[..]);

        if let Ok(cipher_text) = cipher.encrypt(nonce, plain_text) {
            self.encrypt_nonce += 1;

            log::debug!("encrypted cipher_text = {}", utils::fmt::ToHex(cipher_text.clone()));

            Ok(cipher_text.into())
        } else {
            Err(Error::msg("encrypt failed"))
        }
    }

    fn decrypt(&mut self, cipher_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.decrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        log::debug!("decrypt nonce = {}", utils::fmt::ToHex(nonce.to_vec()));
        log::debug!("decrypt cipher_text = {}", utils::fmt::ToHex(cipher_text.to_vec()));

        let cipher_text = Payload::from(&cipher_text[..]);

        if let Ok(plain_text) = cipher.decrypt(nonce, cipher_text) {
            self.decrypt_nonce += 1;

            log::debug!("decrypted plain_text = {}", utils::fmt::ToHex(plain_text.to_vec()));

            Ok(plain_text.into())
        } else {
            Err(Error::msg("decrypt failed"))
        }
    }

    fn get_random_bytes_len(&self, chunk_len: usize) -> usize {
        if chunk_len > 1440 {
            return 0;
        }
        let rand = Crypto::random_u8() as usize;
        if chunk_len > 1300 {
            rand % 31
        } else if chunk_len > 900 {
            rand % 127
        } else if chunk_len > 400 {
            rand % 521
        } else {
            rand % 1021
        }
    }

    fn encode(&mut self, buf: Bytes) -> Result<Bytes> {
        let chunks = utils::buffer::get_chunks(buf, MAX_CHUNK_SIZE);
        let mut enc_chunks = vec![];

        for chunk_buf in chunks {
            let mut buf = BytesMut::with_capacity(1 + TAG_SIZE + 255 + 2 + TAG_SIZE + chunk_buf.len() + TAG_SIZE);

            // generate random padding
            let pad_len = self.get_random_bytes_len(chunk_buf.len());
            let pad_buf = Crypto::random_bytes(pad_len);

            // PaddingLen + PaddingLen Tag
            let enc_pad_len = self.encrypt(utils::buffer::num_to_buf_be(pad_len as u64, 1))?;
            buf.put(enc_pad_len);

            // Padding
            buf.put(pad_buf);

            // ChunkLen + ChunkLen Tag
            let chunk_len = chunk_buf.len();
            let enc_chunk_len = self.encrypt(utils::buffer::num_to_buf_be(chunk_len as u64, 2))?;
            buf.put(enc_chunk_len);

            // Chunk + Chunk Tag
            let enc_chunk = self.encrypt(chunk_buf.clone())?;
            buf.put(enc_chunk);

            enc_chunks.push(buf.freeze());
        }

        Ok(Bytes::from(enc_chunks.concat()))
    }

    async fn decode(&mut self, socket: &Socket) -> Result<Bytes> {
        // PaddingLen
        let enc_pad_len = socket.read_exact(1 + TAG_SIZE).await?;
        let pad_len = self.decrypt(enc_pad_len)?;
        let pad_len = u8::from_be_bytes([pad_len[0]]);

        // Padding
        let _ = socket.read_exact(pad_len as usize).await?;

        // ChunkLen
        let enc_chunk_len = socket.read_exact(2 + TAG_SIZE).await?;
        let chunk_len = self.decrypt(enc_chunk_len)?;
        let chunk_len = u16::from_be_bytes([chunk_len[0], chunk_len[1]]);

        // Chunk
        let enc_chunk = socket.read_exact(chunk_len as usize + TAG_SIZE).await?;
        self.decrypt(enc_chunk)
    }
}

#[async_trait]
impl Protocol for Erp {
    fn get_name(&self) -> String {
        "erp".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> &ResolvedResult {
        self.resolved_result.as_ref().unwrap()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<&ResolvedResult> {
        let salt = socket.read_exact(SALT_SIZE).await?;
        self.derived_key = Some(Self::derive_key(self.raw_key.clone(), salt));

        let chunk = self.decode(socket).await?;

        let (address, pending_buf) = Address::from_bytes(chunk)?;

        self.set_resolved_result(ResolvedResult {
            protocol: ProtocolType::Erp,
            address,
            pending_buf,
        });

        Ok(self.get_resolved_result())
    }

    async fn client_encode(&mut self, socket: &Socket) -> Result<Bytes> {
        let buf = socket.read_some().await?;
        let mut data = BytesMut::with_capacity(buf.len() + 200);

        // attach header
        if !self.header_sent {
            let resolved = self.get_resolved_result();
            data.put(resolved.address.as_bytes());
        }

        data.put(buf);

        let data = self.encode(data.freeze())?;

        if self.header_sent {
            Ok(data)
        } else {
            let mut buf = BytesMut::with_capacity(SALT_SIZE + data.len());

            buf.put(self.salt.as_ref().unwrap().clone());
            buf.put(data);

            self.header_sent = true;

            Ok(buf.freeze())
        }
    }

    async fn server_encode(&mut self, socket: &Socket) -> Result<Bytes> {
        let buf = socket.read_some().await?;
        self.encode(buf)
    }

    async fn client_decode(&mut self, socket: &Socket) -> Result<Bytes> {
        self.decode(socket).await
    }

    async fn server_decode(&mut self, socket: &Socket) -> Result<Bytes> {
        self.decode(socket).await
    }
}
