use net::connection::ConnectionSnapshot;
use std::{collections::HashMap, str::FromStr};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

mod config;
mod event;
mod protocol;

pub mod net;
pub mod utils;

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

#[derive(Debug, Clone)]
pub enum TransportProtocol {
    Plain,
    EncryptRandomPadding,
}

impl FromStr for TransportProtocol {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "erp" => Ok(Self::EncryptRandomPadding),
            _ => Err(format!("{} is not supported, available protocols are: plain, erp", s)),
        }
    }
}

#[derive(Default)]
pub struct SharedData {
    pub conns: HashMap<usize, ConnectionSnapshot>,
}
