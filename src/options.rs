use crate::{net::address::NetAddr, ServiceType};
use clap::{crate_version, Clap};
use std::str::FromStr;

/// Lightweight and efficient proxy written in pure Rust
#[derive(Clap, Debug, Clone)]
#[clap(version = crate_version!(), author = "Micooz Lee <micooz@hotmail.com>")]
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

    /// local service host, default 127.0.0.1
    #[clap(short, default_value = "127.0.0.1")]
    pub host: String,

    /// local service port, default 1080
    #[clap(short, default_value = "1080")]
    pub port: u16,

    /// remote service host, client only
    #[clap(long)]
    pub remote_host: Option<String>,

    /// remote service port, client only
    #[clap(long)]
    pub remote_port: Option<u16>,

    /// protocol used for transport layer between client and server
    #[clap(long)]
    pub protocol: Option<Protocol>,
}

impl Options {
    /// Return combination of host and port
    pub fn get_local_addr(&self) -> NetAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }

    /// Return combination of remote_host and remote_port
    pub fn get_remote_addr(&self) -> NetAddr {
        format!(
            "{}:{}",
            self.remote_host.as_ref().unwrap(),
            self.remote_port.unwrap()
        )
        .parse()
        .unwrap()
    }

    /// Return local service type
    pub fn get_service_type(&self) -> Result<ServiceType, String> {
        if !self.server && self.client {
            return Ok(ServiceType::Client);
        }
        if self.server && !self.client {
            return Ok(ServiceType::Server);
        }
        Err("cannot determine service type".into())
    }
}

#[derive(Debug, Clone)]
pub enum Protocol {
    Plain,
    EncryptRandomPadding,
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "plain" => Ok(Protocol::Plain),
            "erp" => Ok(Protocol::EncryptRandomPadding),
            _ => Err(format!(
                "{} is not supported, available protocols are: plain, erp",
                s
            )),
        }
    }
}
