use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

#[async_trait]
pub trait Protocol {
    fn get_name(&self) -> String;

    async fn parse_proxy_address(
        &self,
        reader: &mut ReadHalf<TcpStream>,
        writer: &mut WriteHalf<TcpStream>,
    ) -> Result<SocketAddr>;

    async fn encode_data(&self, buf: Bytes) -> Result<Bytes>;

    // async fn decode_data(&self, buf: Bytes) -> Result<()>;
}
