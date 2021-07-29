use crate::{net::address::NetAddr, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

pub mod erp;
pub mod plain;
pub mod socks5;
pub mod transparent;

#[async_trait]
pub trait Protocol {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr>;

    fn pack(&mut self, buf: Bytes) -> Result<Bytes>;

    fn unpack(&mut self, buf: Bytes) -> Result<Bytes>;
}
