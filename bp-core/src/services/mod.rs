use std::net::SocketAddr;

use serde::Serialize;

pub mod monitor;
pub mod pac;
pub mod quic;
pub mod tcp;
pub mod tls;
pub mod udp;

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceProtocol {
    Tcp,
    Udp,
    Tls,
    Pac,
    Quic,
    Monitor,
}

impl Serialize for ServiceProtocol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ServiceProtocol::Tcp => "tcp",
            ServiceProtocol::Udp => "udp",
            ServiceProtocol::Tls => "tls",
            ServiceProtocol::Pac => "pac",
            ServiceProtocol::Quic => "quic",
            ServiceProtocol::Monitor => "monitor",
        };
        serializer.serialize_str(s)
    }
}

#[derive(Debug)]
pub enum Startup {
    Success(Vec<ServiceInfo>),
    Fail(anyhow::Error),
}

impl Startup {
    pub fn services(&self) -> Vec<ServiceInfo> {
        match self {
            Startup::Success(info) => info.clone(),
            Startup::Fail(err) => panic!("{}", err),
        }
    }

    pub fn get(&self, protocol: ServiceProtocol) -> Option<ServiceInfo> {
        self.services().into_iter().find(|item| item.protocol == protocol)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub protocol: ServiceProtocol,
    pub bind_addr: SocketAddr,
    pub bind_host: String,
    pub bind_ip: String,
    pub bind_port: u16,
}
