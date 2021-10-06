use crate::{event, net, net::socket, Result};
use async_trait::async_trait;
use std::str;

mod direct;
mod erp;
mod http;
mod https;
mod plain;
mod socks;
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

    pub pending_buf: Option<bytes::Bytes>,
}

#[async_trait]
pub trait Protocol: dyn_clone::DynClone {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<ResolvedResult>;

    fn set_proxy_address(&mut self, _addr: net::Address) {}

    fn get_proxy_address(&self) -> Option<net::Address> {
        unimplemented!()
    }

    async fn client_encode(&mut self, socket: &socket::Socket, tx: event::EventSender) -> Result<()>;

    async fn server_encode(&mut self, socket: &socket::Socket, tx: event::EventSender) -> Result<()>;

    async fn client_decode(&mut self, socket: &socket::Socket, tx: event::EventSender) -> Result<()>;

    async fn server_decode(&mut self, socket: &socket::Socket, tx: event::EventSender) -> Result<()>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub enum TransportProtocol {
    Plain,
    EncryptRandomPadding,
}

impl str::FromStr for TransportProtocol {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "erp" => Ok(Self::EncryptRandomPadding),
            _ => Err(format!("{} is not supported, available protocols are: plain, erp", s)),
        }
    }
}
