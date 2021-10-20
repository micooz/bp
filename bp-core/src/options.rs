use crate::{net::address::Address, protocol::TransportProtocol};

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

    /// bp server bind address, client only. If not set, bp will relay directly
    #[clap(long)]
    pub server_bind: Option<Address>,

    /// symmetric encryption key
    #[clap(short, long)]
    pub key: Option<String>,

    /// protocol used by transport layer between client and server,
    /// "plain" or "erp" are supported
    #[clap(short, long, default_value = "erp")]
    pub protocol: TransportProtocol,

    /// check white list before proxy, pass a file path
    #[clap(long)]
    pub proxy_white_list: Option<String>,

    /// force all incoming data relay to this destination, usually for testing [default: false]
    #[clap(long)]
    pub force_dest_addr: Option<Address>,

    /// proxy UDP via TCP, client only. Requires --server-bind to be set if true [default: false]
    #[clap(long)]
    pub udp_over_tcp: bool,

    /// DNS server address [default: 8.8.8.8:53]
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

    pub fn get_dns_server(&self) -> Address {
        self.dns_server.clone().unwrap_or_else(|| "8.8.8.8:53".parse().unwrap())
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

    // check --proxy-white-list
    if opts.server && opts.proxy_white_list.is_some() {
        return Err("--proxy-white-list can only be set on client.");
    }

    // check --udp-over-tcp
    if opts.udp_over_tcp {
        if opts.server {
            return Err("--udp-over-tcp can only be set on client.");
        }
        if opts.client && opts.server_bind.is_none() {
            return Err("--udp-over-tcp requires --server-bind to be set.");
        }
    }

    // check --force-dest-addr
    if opts.force_dest_addr.is_some() && !opts.client {
        return Err("--force-dest-addr can only be set on client.");
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub enum ServiceType {
    Client,
    Server,
}

impl ServiceType {
    pub fn is_client(&self) -> bool {
        matches!(self, ServiceType::Client)
    }
    pub fn is_server(&self) -> bool {
        matches!(self, ServiceType::Server)
    }
}
