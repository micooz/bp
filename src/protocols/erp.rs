use crate::utils::fmt::ToHex;
use crate::{net::address::NetAddr, utils, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use chacha20poly1305::aead::{Aead, NewAead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use tokio::io::AsyncReadExt;

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
/// * Nonce is little-endian and counting from 0, each chunk increases the nonce twice.
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
pub struct Erp {
    header_sent: bool,

    raw_key: String,

    derived_key: Option<Bytes>,

    encrypt_nonce: u64,

    decrypt_nonce: u64,

    proxy_address: Option<NetAddr>,
}

impl Erp {
    pub fn new(key: String) -> Self {
        Self {
            raw_key: key,
            derived_key: None,
            encrypt_nonce: 0,
            decrypt_nonce: 0,
            header_sent: false,
            proxy_address: None,
        }
    }

    fn derive_key(&mut self, salt: Bytes) {
        if self.derived_key.is_some() {
            return;
        }
        self.derived_key = Some(utils::crypto::hkdf_sha256(
            self.raw_key.clone().into(),
            salt,
            HKDF_INFO.as_bytes().into(),
            KEY_SIZE,
        ));
    }

    fn encrypt(&mut self, plain_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(&self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.encrypt_nonce as u128, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        log::debug!("encrypt key = {}", ToHex(key.to_vec()));
        log::debug!("encrypt nonce = {}", ToHex(nonce.to_vec()));
        log::debug!("encrypt plain_text = {}", ToHex(plain_text.to_vec()));

        let plain_text = Payload::from(&plain_text[..]);

        if let Ok(cipher_text) = cipher.encrypt(nonce, plain_text) {
            self.encrypt_nonce += 1;

            log::debug!("encrypted cipher_text = {}", ToHex(cipher_text.clone()));

            Ok(cipher_text.into())
        } else {
            Err("encrypt failed".into())
        }
    }

    fn decrypt(&mut self, cipher_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(&self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.decrypt_nonce as u128, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        log::debug!("decrypt key = {}", ToHex(key.to_vec()));
        log::debug!("decrypt nonce = {}", ToHex(nonce.to_vec()));
        log::debug!("decrypt cipher_text = {}", ToHex(cipher_text.to_vec()));

        let cipher_text = Payload::from(&cipher_text[..]);

        if let Ok(plain_text) = cipher.decrypt(nonce, cipher_text) {
            self.decrypt_nonce += 1;

            log::debug!("decrypted plain_text = {}", ToHex(plain_text.to_vec()));

            Ok(plain_text.into())
        } else {
            Err("decrypt failed".into())
        }
    }

    fn get_random_bytes_len(&self, chunk_len: usize) -> usize {
        if chunk_len > 1440 {
            return 0;
        }
        let rand = utils::crypto::random_u8() as usize;
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

    fn encode(&mut self, buf: Bytes) -> Bytes {
        let chunks = utils::buffer::get_chunks(buf, MAX_CHUNK_SIZE);

        let chunks: Vec<Bytes> = chunks
            .iter()
            .map(|chunk_buf| {
                let mut buf = BytesMut::new();

                // generate random padding
                let pad_len = self.get_random_bytes_len(chunk_buf.len());
                let pad_buf = utils::crypto::random_bytes(pad_len);

                // prepare chunk
                let chunk_len = chunk_buf.len();

                // PaddingLen + PaddingLen Tag
                let enc_pad_len = self
                    .encrypt(utils::buffer::num_to_buf_be(pad_len as u64, 1))
                    .unwrap();
                buf.put(enc_pad_len);

                // Padding
                buf.put(pad_buf);

                // ChunkLen + ChunkLen Tag
                let enc_chunk_len = self
                    .encrypt(utils::buffer::num_to_buf_be(chunk_len as u64, 2))
                    .unwrap();
                buf.put(enc_chunk_len);

                // Chunk + Chunk Tag
                let enc_chunk = self.encrypt(chunk_buf.clone()).unwrap();
                buf.put(enc_chunk);

                buf.freeze()
            })
            .collect();

        Bytes::from(chunks.concat())
    }

    fn decode(&mut self, mut buf: Bytes) -> Result<Bytes> {
        // PaddingLen
        let enc_pad_len = buf.slice(0..(1 + TAG_SIZE));

        let pad_len = self.decrypt(enc_pad_len)?;
        let pad_len = u8::from_be_bytes([pad_len[0]]);

        buf.advance(1 + TAG_SIZE);

        // Padding
        buf.advance(pad_len as usize);

        // ChunkLen
        let enc_chunk_len = buf.slice(0..(2 + TAG_SIZE));

        let chunk_len = self.decrypt(enc_chunk_len)?;
        let chunk_len = u16::from_be_bytes([chunk_len[0], chunk_len[1]]);

        buf.advance(2 + TAG_SIZE);

        // Chunk
        let enc_chunk = buf.slice(0..chunk_len as usize + TAG_SIZE);

        self.decrypt(enc_chunk)
    }
}

#[async_trait]
impl Protocol for Erp {
    fn get_name(&self) -> String {
        "erp".into()
    }

    fn set_proxy_address(&mut self, addr: NetAddr) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<NetAddr> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(NetAddr, Option<Bytes>)> {
        // Salt
        let mut salt = vec![0u8; SALT_SIZE];
        reader.read_exact(&mut salt).await?;

        // PaddingLen
        let mut enc_pad_len = vec![0u8; 1 + TAG_SIZE];
        reader.read_exact(&mut enc_pad_len).await?;

        self.derive_key(salt.into());

        let pad_len = self.decrypt(enc_pad_len.into())?;
        let pad_len = u8::from_be_bytes([pad_len[0]]);

        // Padding
        let mut padding = vec![0u8; pad_len as usize];
        reader.read_exact(&mut padding).await?;

        // ChunkLen
        let mut enc_chunk_len = vec![0u8; 2 + TAG_SIZE];
        reader.read_exact(&mut enc_chunk_len).await?;

        let chunk_len = self.decrypt(enc_chunk_len.into())?;
        let chunk_len = u16::from_be_bytes([chunk_len[0], chunk_len[1]]);

        // Chunk
        let mut enc_chunk = vec![0u8; (chunk_len + TAG_SIZE as u16) as usize];
        reader.read_exact(&mut enc_chunk).await?;

        let chunk = self.decrypt(enc_chunk.into())?;

        NetAddr::from_bytes(chunk)
    }

    fn client_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        let mut data = BytesMut::with_capacity(buf.len() + 200);
        let mut salt = Bytes::new();

        if !self.header_sent {
            // generate salt and then derive key
            salt = utils::crypto::random_bytes(SALT_SIZE);
            self.derive_key(salt.clone());

            // attach header
            data.put(self.proxy_address.as_ref().unwrap().as_bytes());
        }

        data.put(buf);

        let data = self.encode(data.freeze());

        if self.header_sent {
            Ok(data)
        } else {
            let mut buf = BytesMut::with_capacity(SALT_SIZE + data.len());

            buf.put(salt);
            buf.put(data);

            self.header_sent = true;

            Ok(buf.freeze())
        }
    }

    fn client_decode(&mut self, buf: Bytes) -> Result<Bytes> {
        self.decode(buf)
    }

    fn server_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(self.encode(buf))
    }

    fn server_decode(&mut self, buf: Bytes) -> Result<Bytes> {
        self.decode(buf)
    }
}
