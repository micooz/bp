use crate::{
    event::{Event, EventSender},
    net::{Address, TcpStreamReader, TcpStreamWriter},
    options::ServiceType,
    protocol::Protocol,
    utils, Result,
};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use chacha20poly1305::{
    aead::{Aead, NewAead, Payload},
    ChaCha20Poly1305, Key, Nonce,
};

const RECV_BUFFER_SIZE: usize = 4 * 1024;
const MAX_CHUNK_SIZE: usize = 0x3FFF;
const SALT_SIZE: usize = 32;
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const HKDF_INFO: &str = "bp-subkey1";

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
pub struct Erp {
    header_sent: bool,

    raw_key: String,

    salt: Option<Bytes>,

    derived_key: Option<Bytes>,

    encrypt_nonce: u128,

    decrypt_nonce: u128,

    proxy_address: Option<Address>,
}

impl Erp {
    pub fn new(key: String, service_type: ServiceType) -> Self {
        let (salt, derived_key) = match service_type {
            ServiceType::Server => (None, None),
            // only client side can generate salt and derived_key
            // generate on server side will take no effect.
            ServiceType::Client => {
                let salt = utils::crypto::random_bytes(SALT_SIZE);
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
            proxy_address: None,
        }
    }

    fn derive_key(raw_key: String, salt: Bytes) -> Bytes {
        let derived_key = utils::crypto::hkdf_sha256(raw_key.into(), salt, HKDF_INFO.as_bytes().into(), KEY_SIZE);
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("encrypt/decrypt key = {}", utils::fmt::ToHex(derived_key.to_vec()));
        }
        derived_key
    }

    fn encrypt(&mut self, plain_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.encrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        if log::log_enabled!(log::Level::Debug) {
            log::debug!("encrypt nonce = {}", utils::fmt::ToHex(nonce.to_vec()));
            log::debug!("encrypt plain_text = {}", utils::fmt::ToHex(plain_text.to_vec()));
        }

        let plain_text = Payload::from(&plain_text[..]);

        if let Ok(cipher_text) = cipher.encrypt(nonce, plain_text) {
            self.encrypt_nonce += 1;

            log::debug!("encrypted cipher_text = {}", utils::fmt::ToHex(cipher_text.clone()));

            Ok(cipher_text.into())
        } else {
            Err("encrypt failed".into())
        }
    }

    fn decrypt(&mut self, cipher_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(self.derived_key.as_ref().unwrap());
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.decrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        if log::log_enabled!(log::Level::Debug) {
            log::debug!("decrypt nonce = {}", utils::fmt::ToHex(nonce.to_vec()));
            log::debug!("decrypt cipher_text = {}", utils::fmt::ToHex(cipher_text.to_vec()));
        }

        let cipher_text = Payload::from(&cipher_text[..]);

        if let Ok(plain_text) = cipher.decrypt(nonce, cipher_text) {
            self.decrypt_nonce += 1;

            log::debug!("decrypted plain_text = {}", utils::fmt::ToHex(plain_text.to_vec()));

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

    fn encode(&mut self, buf: Bytes) -> Result<Bytes> {
        let chunks = utils::buffer::get_chunks(buf, MAX_CHUNK_SIZE);
        let mut enc_chunks = vec![];

        for chunk_buf in chunks {
            let mut buf = BytesMut::new();

            // generate random padding
            let pad_len = self.get_random_bytes_len(chunk_buf.len());
            let pad_buf = utils::crypto::random_bytes(pad_len);

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

    async fn decode(&mut self, reader: &mut TcpStreamReader) -> Result<Bytes> {
        // PaddingLen
        let enc_pad_len = reader.read_exact(1 + TAG_SIZE).await?;
        let pad_len = self.decrypt(enc_pad_len)?;
        let pad_len = u8::from_be_bytes([pad_len[0]]);

        // Padding
        let _ = reader.read_exact(pad_len as usize).await?;

        // ChunkLen
        let enc_chunk_len = reader.read_exact(2 + TAG_SIZE).await?;
        let chunk_len = self.decrypt(enc_chunk_len)?;
        let chunk_len = u16::from_be_bytes([chunk_len[0], chunk_len[1]]);

        // Chunk
        let enc_chunk = reader.read_exact(chunk_len as usize + TAG_SIZE).await?;
        self.decrypt(enc_chunk)
    }
}

impl Clone for Erp {
    fn clone(&self) -> Self {
        Self {
            header_sent: self.header_sent,
            raw_key: self.raw_key.clone(),
            salt: self.salt.clone(),
            derived_key: self.derived_key.clone(),
            encrypt_nonce: self.encrypt_nonce,
            decrypt_nonce: self.decrypt_nonce,
            proxy_address: self.proxy_address.clone(),
        }
    }
}

#[async_trait]
impl Protocol for Erp {
    fn get_name(&self) -> String {
        "erp".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)> {
        let salt = reader.read_exact(SALT_SIZE).await?;
        self.derived_key = Some(Self::derive_key(self.raw_key.clone(), salt));

        let chunk = self.decode(reader).await?;
        Address::from_bytes(chunk)
    }

    async fn client_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let buf = reader.read_buf(RECV_BUFFER_SIZE).await?;
        let mut data = BytesMut::with_capacity(buf.len() + 200);

        // attach header
        if !self.header_sent {
            data.put(self.proxy_address.as_ref().unwrap().as_bytes());
        }

        data.put(buf);

        let data = self.encode(data.freeze())?;

        if self.header_sent {
            tx.send(Event::EncodeDone(data)).await?;
            Ok(())
        } else {
            let mut buf = BytesMut::with_capacity(SALT_SIZE + data.len());

            buf.put(self.salt.as_ref().unwrap().clone());
            buf.put(data);

            self.header_sent = true;

            tx.send(Event::EncodeDone(buf.freeze())).await?;
            Ok(())
        }
    }

    async fn server_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let buf = reader.read_buf(RECV_BUFFER_SIZE).await?;
        let data = self.encode(buf)?;
        tx.send(Event::EncodeDone(data)).await?;
        Ok(())
    }

    async fn client_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        loop {
            let chunk = self.decode(reader).await?;
            tx.send(Event::DecodeDone(chunk)).await?;
        }
    }

    async fn server_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        self.client_decode(reader, tx).await
    }
}
