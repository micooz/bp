use crate::{
    net::{NetAddr, TcpStreamReader, TcpStreamWriter},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;

mod erp;
mod plain;
mod socks5;
mod transparent;

pub use erp::Erp;
pub use plain::Plain;
pub use socks5::Socks5;
pub use transparent::Transparent;

#[async_trait]
pub trait Protocol {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<(NetAddr, Option<Bytes>)>;

    fn set_proxy_address(&mut self, addr: NetAddr);

    fn get_proxy_address(&self) -> Option<NetAddr>;

    fn client_encode(&mut self, buf: Bytes) -> Result<Bytes>;

    fn client_decode(&mut self, buf: Bytes) -> Result<DecodeStatus>;

    fn server_encode(&mut self, buf: Bytes) -> Result<Bytes>;

    fn server_decode(&mut self, buf: Bytes) -> Result<DecodeStatus>;
}

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;

pub enum DecodeStatus {
    Pending,
    Fulfil(Bytes),
}
