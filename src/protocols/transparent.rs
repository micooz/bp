use crate::{
    net::NetAddr,
    protocols::{DecodeStatus, Protocol},
    Result,
};
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

    fn set_proxy_address(&mut self, _addr: NetAddr) {
        // unimplemented!()
    }

    fn get_proxy_address(&self) -> Option<NetAddr> {
        unimplemented!()
    }

    async fn resolve_proxy_address(
        &mut self,
        _reader: &mut ReadHalf<TcpStream>,
        _writer: &mut WriteHalf<TcpStream>,
    ) -> Result<(NetAddr, Option<Bytes>)> {
        unimplemented!("transparent protocol cannot be used on inbound")
    }

    fn client_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn client_decode(&mut self, buf: Bytes) -> Result<DecodeStatus> {
        Ok(DecodeStatus::Fulfil(buf))
    }

    fn server_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn server_decode(&mut self, buf: Bytes) -> Result<DecodeStatus> {
        Ok(DecodeStatus::Fulfil(buf))
    }
}
