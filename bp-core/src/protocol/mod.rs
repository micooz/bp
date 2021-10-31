use crate::{
    net::{address::Address, socket::Socket},
    options::Options,
};
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{de::Visitor, Deserialize, Deserializer};
use std::str::{self, FromStr};

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

impl ResolvedResult {
    pub fn set_port(&mut self, port: u16) {
        self.address.set_port(port);
    }
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

    async fn client_encode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn server_encode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn client_decode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn server_decode(&mut self, socket: &Socket) -> Result<Bytes>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq)]
pub enum TransportProtocol {
    Plain,
    EncryptRandomPadding,
}

impl str::FromStr for TransportProtocol {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "~" | "" | "erp" => Ok(Self::EncryptRandomPadding),
            _ => Err(format!("{} is not supported, available protocols are: plain, erp", s)),
        }
    }
}

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::EncryptRandomPadding
    }
}

struct TransportProtocolVisitor;

impl<'de> Visitor<'de> for TransportProtocolVisitor {
    type Value = TransportProtocol;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("plain/erp")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TransportProtocol::from_str(v).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for TransportProtocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(TransportProtocolVisitor)
    }
}

pub fn init_transport_protocol(opts: &Options) -> DynProtocol {
    match opts.protocol {
        TransportProtocol::Plain => Box::new(Plain::default()),
        TransportProtocol::EncryptRandomPadding => Box::new(Erp::new(opts.key.clone().unwrap(), opts.service_type())),
    }
}
