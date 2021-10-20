use crate::{
    event::EventSender,
    net::{address::Address, socket::Socket},
    options::Options,
    Result,
};
use async_trait::async_trait;
use std::str;

mod direct;
mod dns;
mod erp;
mod http;
mod https;
mod plain;
mod socks;

pub use direct::Direct;
pub use dns::Dns;
pub use erp::Erp;
pub use http::Http;
pub use https::Https;
pub use plain::Plain;
pub use socks::Socks;

#[derive(Debug, Clone)]
pub struct ResolvedResult {
    pub protocol: ProtocolType,

    pub address: Address,

    pub pending_buf: Option<bytes::Bytes>,
}

#[derive(Debug, Clone)]
pub enum ProtocolType {
    Direct,
    Dns,
    Erp,
    Http,
    Https,
    Plain,
    Socks,
}

#[async_trait]
pub trait Protocol: dyn_clone::DynClone {
    fn get_name(&self) -> String;

    fn set_resolved_result(&mut self, _res: ResolvedResult) {
        unimplemented!();
    }

    fn get_resolved_result(&self) -> Option<ResolvedResult> {
        unimplemented!();
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<()>;

    async fn client_encode(&mut self, socket: &Socket, tx: EventSender) -> Result<()>;

    async fn server_encode(&mut self, socket: &Socket, tx: EventSender) -> Result<()>;

    async fn client_decode(&mut self, socket: &Socket, tx: EventSender) -> Result<()>;

    async fn server_decode(&mut self, socket: &Socket, tx: EventSender) -> Result<()>;
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

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::EncryptRandomPadding
    }
}

pub fn init_transport_protocol(opts: &Options) -> DynProtocol {
    match opts.protocol {
        TransportProtocol::Plain => Box::new(Plain::default()),
        TransportProtocol::EncryptRandomPadding => Box::new(Erp::new(opts.key.clone().unwrap(), opts.service_type())),
    }
}
