use super::{NetAddr, Protocol, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

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

    fn pack(&self, buf: Bytes) -> Result<Bytes> {
        todo!()
    }

    fn unpack(&self, buf: Bytes) -> Result<Bytes> {
        todo!()
    }
}
