use crate::{net::address::ProxyAddr, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;

#[async_trait]
pub trait Protocol {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<ProxyAddr>;

    async fn pack(&self, buf: Bytes) -> Result<Bytes>;

    async fn unpack(&self, buf: Bytes) -> Result<Bytes>;
}
