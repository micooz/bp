use crate::{
    event::EventSender,
    net::{address::Address, socket},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;
use dyn_clone::DynClone;

mod direct;
mod erp;
mod http;
mod plain;
mod socks;
mod socks_http;

pub use direct::Direct;
pub use erp::Erp;
pub use http::Http;
pub use plain::Plain;
pub use socks::Socks;
pub use socks_http::SocksHttp;

#[async_trait]
pub trait Protocol: DynClone {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<(Address, Option<Bytes>)>;

    fn set_proxy_address(&mut self, _addr: Address) {}

    fn get_proxy_address(&self) -> Option<Address> {
        unimplemented!()
    }

    async fn client_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn server_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn client_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn server_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;
