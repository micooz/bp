use super::{NetAddr, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

pub struct Plain {}

impl Plain {
    pub fn new() -> Self {
        Plain {}
    }
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr> {
        let header = NetAddr::decode(reader).await?;
        Ok(header)
    }

    fn pack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn unpack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
