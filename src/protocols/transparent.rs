use super::{super::net::address::NetAddr, Protocol, Result};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

pub struct Transparent {}

impl Transparent {
    pub fn new() -> Self {
        Transparent {}
    }
}

#[async_trait]
impl Protocol for Transparent {
    fn get_name(&self) -> String {
        "transparent".into()
    }

    async fn resolve_proxy_address(
        &mut self,
        _reader: &mut ReadHalf<TcpStream>,
        _writer: &mut WriteHalf<TcpStream>,
    ) -> Result<NetAddr> {
        unimplemented!("transparent protocol cannot be used on inbound")
    }

    fn pack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn unpack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
