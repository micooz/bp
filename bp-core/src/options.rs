use crate::net::{Address, ServiceType};
use crate::TransportProtocol;

/// The default local service host
const DEFAULT_SERVICE_ADDRESS: &str = "127.0.0.1:1080";

/// Lightweight and efficient proxy written in pure Rust
#[derive(clap::Parser, Default, Debug, Clone)]
#[clap(name = "bp", version = clap::crate_version!())]
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

    /// enable udp relay, default: false
    #[clap(long)]
    pub enable_udp: bool,

    /// check white list before proxy
    #[clap(long)]
    pub proxy_list_path: Option<String>,

    /// force all incoming data relay to this destination, usually for testing
    #[clap(long)]
    pub force_dest_addr: Option<Address>,

    /// proxy DNS queries via TCP, default: false
    #[clap(long)]
    pub dns_over_tcp: bool,

    /// DNS server address, default: 8.8.8.8:53
    #[clap(long)]
    pub dns_server: Option<Address>,
}

impl Options {
    /// Return local service type
    pub fn service_type(&self) -> ServiceType {
        if !self.server && self.client {
            return ServiceType::Client;
        }
        if self.server && !self.client {
            return ServiceType::Server;
        }
        panic!("cannot determine service type");
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
        return Err("--proxy-list-path can only be set on client.");
    }

    // check --force-dest-addr
    if opts.force_dest_addr.is_some() && !opts.client {
        return Err("--force-dest-addr can only be set on client.");
    }

    Ok(())
}
