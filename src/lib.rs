use protocols::Protocol;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;
type TcpStreamReader = ReadHalf<TcpStream>;
type TcpStreamWriter = WriteHalf<TcpStream>;
type Proto = Box<dyn Protocol + Send + Sync + 'static>;

mod bootstrap;
mod net;
mod protocols;
mod utils;

pub use bootstrap::bootstrap;
pub mod options;

pub enum ServiceType {
    Server,
    Client,
}
