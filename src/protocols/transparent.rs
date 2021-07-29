use crate::{net::address::NetAddr, Protocol, Result};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

pub struct Transparent {}

impl Transparent {
    pub fn new() -> Self {
        Self {}
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

    fn pack(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn unpack(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
