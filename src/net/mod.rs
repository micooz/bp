use std::net::SocketAddr;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

mod address;
mod bound;
mod connection;
mod local;

pub use address::NetAddr;
pub use connection::Connection;
pub use local::bootstrap;

pub type TcpStreamReader = ReadHalf<TcpStream>;
pub type TcpStreamWriter = WriteHalf<TcpStream>;

#[derive(Debug)]
pub struct AcceptResult {
    /// The underlying socket.
    pub socket: TcpStream,

    /// The incoming address.
    pub addr: SocketAddr,
}
