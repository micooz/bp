use super::{NetAddr, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

/// # Protocol
///
/// Header
/// +------+------------+-----+
/// |  IV  | DataFrame  | ... |
/// +------+------------+-----+
/// |  16  |  Variable  | ... |
/// +------+------------+-----+
///
/// DataFrame
/// +------------+------------+-----------+-----------+
/// | PaddingLen |  Padding   |  ChunkLen |   Chunk   |
/// +------------+------------+-----------+-----------+
/// |     1      |  Variable  |     2     | Variable  |
/// +------------+------------+-----------+-----------+
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
/// * AES-128-CTR is used to encrypt all DataFrames
/// * IV is randomly generated
/// * Key derivation function is EVP_BytesToKey
/// * Client Cipher IV = Server Decipher IV, vice versa
///
/// # Reference
///   [1] EVP_BytesToKey
///       https://www.openssl.org/docs/man1.0.2/crypto/EVP_BytesToKey.html
///       https://github.com/shadowsocks/shadowsocks/blob/master/shadowsocks/cryptor.py#L53
pub struct Erp {}

impl Erp {
    pub fn new() -> Self {
        Self {}
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
        todo!()
    }

    fn pack(&self, _buf: Bytes) -> Result<Bytes> {
        todo!()
    }

    fn unpack(&self, _buf: Bytes) -> Result<Bytes> {
        todo!()
    }
}
