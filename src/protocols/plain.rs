use crate::{net::address::NetAddr, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

/// # Protocol
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
pub struct Plain {}

impl Plain {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    fn set_proxy_address(&mut self, _addr: NetAddr) {
        unimplemented!()
    }

    fn get_proxy_address(&self) -> Option<NetAddr> {
        unimplemented!()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(NetAddr, Option<Bytes>)> {
        let header = NetAddr::from_reader(reader).await?;
        Ok((header, None))
    }

    fn client_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn client_decode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn server_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn server_decode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
