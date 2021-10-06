use crate::{event::EventSender, net, net::socket, Result};
use async_trait::async_trait;
use bytes::Bytes;
use dyn_clone::DynClone;

mod direct;
mod erp;
mod http;
mod plain;
mod socks;
mod https;
mod universal;

pub use direct::Direct;
pub use erp::Erp;
pub use http::Http;
pub use https::Https;
pub use plain::Plain;
pub use socks::Socks;
pub use universal::Universal;

#[derive(Debug)]
pub struct ResolvedResult {
    pub protocol: String,

    pub address: net::Address,

    pub pending_buf: Option<Bytes>,
}

#[async_trait]
pub trait Protocol: DynClone {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<ResolvedResult>;

    fn set_proxy_address(&mut self, _addr: net::Address) {}

    fn get_proxy_address(&self) -> Option<net::Address> {
        unimplemented!()
    }

    async fn client_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn server_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn client_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;

    async fn server_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;
