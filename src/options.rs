use crate::{
    config::{CRATE_AUTHOR, DEFAULT_SERVICE_ADDRESS},
    net::Address,
    protocol::{DynProtocol, Erp, Plain},
};
use clap::{crate_version, Clap};
use std::str::FromStr;

/// Lightweight and efficient proxy written in pure Rust
#[derive(Clap, Debug, Clone)]
#[clap(version = crate_version!(), author = CRATE_AUTHOR)]
pub struct Options {
    /// run as server
    #[clap(short)]
    pub server: bool,

    /// run as client
    #[clap(short)]
    pub client: bool,

    /// symmetric encryption key
    #[clap(short)]
    pub key: String,

    /// local service bind address host
    #[clap(short, long, default_value = DEFAULT_SERVICE_ADDRESS)]
    pub bind: String,

    /// bp server host, client only,
    /// if not set, bp will run as transparent proxy
    #[clap(long)]
    pub server_host: Option<String>,

    /// bp server port, client only,
    /// if not set, bp will run as transparent proxy
    #[clap(long)]
    pub server_port: Option<u16>,

    /// protocol used for transport layer between client and server,
    /// "plain" or "erp" are supported.
    #[clap(long, default_value = "erp")]
    pub protocol: Protocol,
}

impl Options {
    /// Return combination of remote_host and remote_port
    pub fn get_server_addr(&self) -> Result<Address, &'static str> {
        format!("{}:{}", self.server_host.as_ref().unwrap(), self.server_port.unwrap()).parse()
    }

    /// Return wether a transparent proxy
    pub fn is_transparent_proxy(&self) -> bool {
        self.server_host.is_none() || self.server_port.is_none()
    }

    /// Return local service type
    pub fn get_service_type(&self) -> Result<ServiceType, &'static str> {
        if !self.server && self.client {
            return Ok(ServiceType::Client);
        }
        if self.server && !self.client {
            return Ok(ServiceType::Server);
        }
        Err("cannot determine service type")
    }

    /// Return initialize protocol object
    pub fn init_protocol(&self) -> Result<DynProtocol, &'static str> {
        let proto: DynProtocol = match self.protocol {
            Protocol::Plain => Box::new(Plain::new()),
            Protocol::EncryptRandomPadding => Box::new(Erp::new(self.key.clone(), self.get_service_type()?)),
        };
        Ok(proto)
    }
}

#[derive(Debug, Clone)]
pub enum Protocol {
    Plain,
    EncryptRandomPadding,
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Protocol::Plain),
            "erp" => Ok(Protocol::EncryptRandomPadding),
            _ => Err(format!("{} is not supported, available protocols are: plain, erp", s)),
        }
    }
}

pub enum ServiceType {
    Server,
    Client,
}
