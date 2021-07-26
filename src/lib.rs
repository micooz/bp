use protocols::proto::Protocol;
use tokio::{
    io::{ReadHalf, WriteHalf},
    net::TcpStream,
};

pub mod net;
pub mod options;
pub mod protocols;
pub mod utils;

pub enum ServiceType {
    Server,
    Client,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub type TcpSteamReader = ReadHalf<TcpStream>;
pub type TcpStreamWriter = WriteHalf<TcpStream>;
pub type Proto = Box<dyn Protocol + Send + Sync + 'static>;
