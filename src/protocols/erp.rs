use crate::{net::address::NetAddr, utils, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use chacha20poly1305::aead::{Aead, NewAead, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};

const MAX_CHUNK_SIZE: usize = 0x3FFF;
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const HKDF_INFO: &'static str = "bp-subkey";

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

    salt: Bytes,

    derived_key: Bytes,

    encrypt_nonce: u64,

    decrypt_nonce: u64,

    proxy_address: Option<NetAddr>,
}

impl Erp {
    pub fn new(key: String) -> Self {
        let salt = utils::crypto::random_bytes(32);
        let derived_key = utils::crypto::hkdf_sha256(
            Bytes::from(key),
            salt.clone(),
            Bytes::from(HKDF_INFO.as_bytes()),
            KEY_SIZE,
        );

        Self {
            salt,
            derived_key,
            encrypt_nonce: 0,
            decrypt_nonce: 0,
            header_sent: false,
            proxy_address: None,
        }
    }

    fn encrypt(&mut self, plain_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(&self.derived_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.encrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        let plain_text = Payload::from(&plain_text[..]);

        if let Ok(cipher_text) = cipher.encrypt(nonce, plain_text) {
            self.encrypt_nonce += 1;
            Ok(cipher_text.into())
        } else {
            Err("encrypt failed".into())
        }
    }

    fn decrypt(&mut self, cipher_text: Bytes) -> Result<Bytes> {
        let key = Key::from_slice(&self.derived_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = utils::buffer::num_to_buf_le(self.encrypt_nonce, NONCE_SIZE);
        let nonce = Nonce::from_slice(&nonce);

        let cipher_text = Payload::from(&cipher_text[..]);

        if let Ok(plain_text) = cipher.decrypt(nonce, cipher_text) {
            Ok(plain_text.into())
        } else {
            Err("encrypt failed".into())
        }
    }
}

#[async_trait]
impl Protocol for Erp {
    fn get_name(&self) -> String {
        "erp".into()
    }

    async fn resolve_proxy_address(
        &mut self,
        _reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr> {
        // // check the first two bytes
        // let mut buf = vec![0u8; 2];
        // reader.read_exact(&mut buf).await?;
        todo!()
    }

    fn pack(&mut self, buf: Bytes) -> Result<Bytes> {
        let mut data = BytesMut::with_capacity(buf.len() + 200);

        let salt_sent = self.header_sent;

        if !self.header_sent {
            let header = self.proxy_address.as_ref().unwrap();
            data.put(header.as_bytes());
            self.header_sent = true;
        }

        data.put(buf);

        let chunks = utils::buffer::get_chunks(data.freeze(), MAX_CHUNK_SIZE);

        let chunks: Vec<Bytes> = chunks
            .iter()
            .map(|chunk_buf| {
                let mut buf = BytesMut::new();

                // generate random padding
                let pad_len = utils::crypto::random_u8();
                let pad_buf = utils::crypto::random_bytes(pad_len as usize);

                // prepare chunk
                let chunk_len = chunk_buf.len();

                // PaddingLen + PaddingLen Tag
                let enc_pad_len = self
                    .encrypt(utils::buffer::num_to_buf_be(pad_len as u64, 2))
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

        let data = chunks.concat();

        if salt_sent {
            Ok(data.into())
        } else {
            let mut buf = BytesMut::with_capacity(32 + data.len());

            buf.put(self.salt.clone());
            buf.put(Bytes::from(data));

            Ok(buf.freeze())
        }
    }

    fn unpack(&mut self, _buf: Bytes) -> Result<Bytes> {
        todo!()
    }
}
