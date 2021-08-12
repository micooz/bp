use std::net::SocketAddr;
use tokio::net::TcpStream;

mod address;
mod bound;
mod connection;
mod io;
mod local;

pub use address::{Address, Host};
pub use connection::Connection;
pub use io::{TcpStreamReader, TcpStreamWriter};
pub use local::bootstrap;

#[derive(Debug)]
pub struct AcceptResult {
    /// The underlying socket.
    pub socket: TcpStream,

    /// The incoming address.
    pub addr: SocketAddr,
}
