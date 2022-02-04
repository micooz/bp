use std::net::SocketAddr;

pub mod pac;
pub mod quic;
pub mod tcp;
pub mod tls;
pub mod udp;

#[derive(Debug)]
pub struct StartupInfo {
    pub bind_addr: SocketAddr,
    pub bind_ip: String,
    pub bind_host: String,
    pub bind_port: u16,
}
