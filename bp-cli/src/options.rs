use bp_core::net;
use bp_core::TransportProtocol;
use bp_core::net::Address;
use clap::{crate_version, Clap};

/// The crate author
const CRATE_AUTHOR: &str = "Micooz Lee <micooz@hotmail.com>";

/// The default local service host
const DEFAULT_SERVICE_ADDRESS: &str = "127.0.0.1:1080";

/// Lightweight and efficient proxy written in pure Rust
#[derive(Clap, Default, Debug, Clone)]
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
    pub bind: Address,

    /// bp server bind address, client only. if not set, bp will relay directly
    #[clap(long)]
    pub server_bind: Option<Address>,

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

    /// force all incoming data relay to this destination, usually for testing
    #[clap(long)]
    pub force_dest_addr: Option<Address>,
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

pub fn check_options(opts: &Options) -> Result<(), &'static str> {
    if !opts.client && !opts.server {
        return Err("--c or --s must be set.");
    }

    if opts.client && opts.server {
        return Err("-c or -s can only be set one.");
    }

    // check --server-bind
    if opts.client && opts.server_bind.is_none() {
        log::warn!("--server-host or --server-port not set, bp will relay directly.");
    }

    // check --key
    if opts.key.is_none() && (opts.server_bind.is_some() || opts.server) {
        return Err("-k or --key must be set.");
    }

    // check --proxy-list-path
    if opts.server && opts.proxy_list_path.is_some() {
        return Err("--proxy-list-path can only be used on client");
    }

    Ok(())
}
