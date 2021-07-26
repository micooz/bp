use super::proto::Protocol;
use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

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

    async fn parse_proxy_address(
        &self,
        _reader: &mut ReadHalf<TcpStream>,
        _writer: &mut WriteHalf<TcpStream>,
    ) -> Result<SocketAddr> {
        Ok("127.0.0.1:8080".parse().unwrap())
    }

    async fn encode_data(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
