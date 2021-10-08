use crate::{net, protocol};

pub mod address;
pub mod connection;
pub mod dns;
pub mod inbound;
pub mod io;
pub mod outbound;
pub mod service;
pub mod socket;

#[cfg(target_os = "linux")]
pub mod linux;

pub use address::Address;
pub use connection::Connection;

#[derive(Clone)]
pub struct ConnOptions {
    pub id: usize,
    pub service_type: ServiceType,
    pub protocol: protocol::TransportProtocol,
    pub key: Option<String>,
    pub local_addr: net::Address,
    pub server_addr: Option<net::Address>,
    pub enable_white_list: bool,
}

#[derive(Clone, Copy)]
pub enum ServiceType {
    Client,
    Server,
}

impl ServiceType {
    fn is_client(&self) -> bool {
        matches!(self, ServiceType::Client)
    }
    fn is_server(&self) -> bool {
        matches!(self, ServiceType::Server)
    }
}
