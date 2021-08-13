use crate::{
    event::EventSender,
    net::{Address, TcpStreamReader, TcpStreamWriter},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;
use dyn_clone::DynClone;

mod erp;
mod http;
mod plain;
mod socks;
mod socks_http;
mod transparent;

pub use erp::Erp;
pub use http::Http;
pub use plain::Plain;
pub use socks::Socks;
pub use socks_http::SocksHttp;
pub use transparent::Transparent;

#[async_trait]
pub trait Protocol: DynClone {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)>;

    fn set_proxy_address(&mut self, addr: Address);

    fn get_proxy_address(&self) -> Option<Address>;

    async fn client_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn server_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn client_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn server_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;
