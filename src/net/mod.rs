use bytes::Bytes;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub mod address;
pub mod bound;
pub mod connection;
pub mod service;

mod context;

#[derive(Debug)]
pub struct AcceptResult {
    /// The underlying socket.
    pub socket: TcpStream,

    /// The incoming address.
    pub addr: SocketAddr,
}

#[derive(Debug)]
pub enum ConnectionEvent {
    InboundRecv(Bytes),
    InboundClose,
    OutboundRecv(Bytes),
    OutboundClose,
}
