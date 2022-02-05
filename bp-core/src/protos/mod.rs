use std::str::{self, FromStr};

use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::{
    net::{address::Address, socket::Socket},
    ServiceType,
};

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
    pub pending_buf: Option<Bytes>,
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
    HttpProxy,
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

    fn get_resolved_result(&self) -> &ResolvedResult {
        unimplemented!();
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<&ResolvedResult>;

    async fn client_encode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn server_encode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn client_decode(&mut self, socket: &Socket) -> Result<Bytes>;

    async fn server_decode(&mut self, socket: &Socket) -> Result<Bytes>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncryptionMethod {
    Plain,
    EncryptRandomPadding,
}

impl str::FromStr for EncryptionMethod {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "~" | "" | "erp" => Ok(Self::EncryptRandomPadding),
            _ => Err(format!("{} is not supported, available methods are: plain, erp", s)),
        }
    }
}

impl Default for EncryptionMethod {
    fn default() -> Self {
        Self::EncryptRandomPadding
    }
}

impl ToString for EncryptionMethod {
    fn to_string(&self) -> String {
        match self {
            Self::Plain => "plain".to_string(),
            Self::EncryptRandomPadding => "erp".to_string(),
        }
    }
}

impl Serialize for EncryptionMethod {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct EncryptionMethodVisitor;

impl<'de> Visitor<'de> for EncryptionMethodVisitor {
    type Value = EncryptionMethod;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("plain/erp")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        EncryptionMethod::from_str(v).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for EncryptionMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(EncryptionMethodVisitor)
    }
}

pub fn init_protocol(encryption: EncryptionMethod, key: String, service_type: ServiceType) -> DynProtocol {
    match encryption {
        EncryptionMethod::Plain => Box::new(Plain::default()),
        EncryptionMethod::EncryptRandomPadding => Box::new(Erp::new(key, service_type)),
    }
}
