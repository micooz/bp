use super::proto::Protocol;
use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
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

    async fn parse_proxy_address(
        &self,
        _reader: &mut ReadHalf<TcpStream>,
        _writer: &mut WriteHalf<TcpStream>,
    ) -> Result<SocketAddr> {
        unimplemented!("transparent protocol cannot be used on inbound")
    }

    async fn encode_data(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
