use bp_lib::net;
use bp_lib::TransportProtocol;
use clap::{crate_version, Clap};

/// The crate author
const CRATE_AUTHOR: &str = "Micooz Lee <micooz@hotmail.com>";

/// The default local service host
const DEFAULT_SERVICE_ADDRESS: &str = "127.0.0.1:1080";

/// Lightweight and efficient proxy written in pure Rust
#[derive(Clap, Debug, Clone)]
#[clap(version = crate_version!(), author = CRATE_AUTHOR)]
pub struct Options {
    /// run as server
    #[clap(short, long)]
    pub server: bool,

    /// run as client
    #[clap(short, long)]
    pub client: bool,

    /// local service bind address
    #[clap(short, long, default_value = DEFAULT_SERVICE_ADDRESS)]
    pub bind: String,

    /// bp server host, client only. if not set, bp will relay directly
    #[clap(long)]
    pub server_host: Option<String>,

    /// bp server port, client only. if not set, bp will relay directly
    #[clap(long)]
    pub server_port: Option<u16>,

    /// symmetric encryption key
    #[clap(short, long)]
    pub key: Option<String>,

    /// protocol used by transport layer between client and server,
    /// "plain" or "erp" are supported.
    #[clap(long, default_value = "erp")]
    pub protocol: TransportProtocol,

    /// enable udp relay
    #[clap(long)]
    pub enable_udp: bool,

    /// check white list before proxy
    #[clap(long)]
    pub proxy_list_path: Option<String>,
}

impl Options {
    /// Return local service type
    pub fn get_service_type(&self) -> Result<net::ServiceType, &'static str> {
        if !self.server && self.client {
            return Ok(net::ServiceType::Client);
        }
        if self.server && !self.client {
            return Ok(net::ServiceType::Server);
        }
        Err("cannot determine service type")
    }

    #[cfg(feature = "monitor")]
    /// Return monitor bind address
    pub fn get_monitor_bind_addr(&self) -> String {
        use bp_lib::net::address::Address;

        let mut addr: Address = self.bind.parse().unwrap();
        addr.set_port(addr.port + 1);

        addr.as_string()
    }
}
