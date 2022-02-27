use std::net::SocketAddr;

use serde::Serialize;
// use tokio::task::JoinHandle;

pub mod monitor;
pub mod pac;
pub mod quic;
pub mod tcp;
pub mod tls;
pub mod udp;

#[derive(Debug)]
pub enum Startup {
    Success(ServiceInfo),
    Fail(anyhow::Error),
}

impl Startup {
    pub fn info(&self) -> ServiceInfo {
        match self {
            Startup::Success(info) => info.clone(),
            Startup::Fail(err) => panic!("{}", err),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub bind_addr: SocketAddr,
    pub bind_ip: String,
    pub bind_host: String,
    pub bind_port: u16,
}
