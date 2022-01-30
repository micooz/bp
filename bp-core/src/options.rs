use std::{fs, io::Read};

use anyhow::{Error, Result};

use crate::{
    config::{DEFAULT_DNS_SERVER_ADDRESS, DEFAULT_SERVICE_ADDRESS},
    net::address::Address,
    proto::ApplicationProtocol,
};

/// Lightweight and efficient proxy written in pure Rust
#[derive(clap::Parser, serde::Deserialize, Default, Debug, Clone)]
#[clap(name = "bp", version = clap::crate_version!())]
pub struct Options {
    /// Configuration file in YAML/JSON format
    #[clap(long)]
    #[serde(skip)]
    pub config: Option<String>,

    /// run as server
    #[clap(short, long)]
    #[serde(default)]
    pub server: bool,

    /// run as client
    #[clap(short, long)]
    #[serde(default)]
    pub client: bool,

    // run as daemon process, unix only
    // #[clap(short, long)]
    // #[serde(default)]
    // pub daemonize: bool,

    /// local service bind address
    #[clap(short, long, default_value = DEFAULT_SERVICE_ADDRESS)]
    pub bind: Address,

    /// bp server bind address, client only. If not set, bp will relay directly
    #[clap(long)]
    pub server_bind: Option<Address>,

    /// symmetric encryption key
    #[clap(short, long)]
    pub key: Option<String>,

    /// protocol used between client and server, "plain" or "erp" are supported
    #[clap(short, long, default_value = "erp")]
    #[serde(default)]
    pub protocol: ApplicationProtocol,

    /// check white list before proxy, pass a file path
    #[clap(long)]
    pub proxy_white_list: Option<String>,

    /// force all incoming data relay to this destination, usually for testing
    #[clap(long)]
    pub force_dest_addr: Option<Address>,

    /// proxy UDP via TCP, client only. Requires --server-bind to be set if true [default: false]
    #[clap(long)]
    #[serde(default)]
    pub udp_over_tcp: bool,

    /// DNS server address [default: 8.8.8.8:53]
    #[clap(long)]
    pub dns_server: Option<Address>,

    /// Enable QUIC to transfer data
    #[clap(long)]
    #[serde(default)]
    pub quic: bool,

    /// the max number of QUIC connections [default: 65535]
    #[clap(long)]
    #[serde(default)]
    pub quic_max_concurrency: Option<u16>,

    /// generate TLS certificate and key files(in der format) to CWD
    #[clap(long)]
    #[serde(default)]
    pub generate_cert: bool,

    /// certificate file for Quic or TLS
    #[clap(long)]
    pub tls_cert: Option<String>,

    /// private key file for Quic or TLS, server only
    #[clap(long)]
    pub tls_key: Option<String>,
}

impl Options {
    pub fn from_file(file: &str) -> Result<Self> {
        let mut raw_str = String::new();
        let mut fd = fs::OpenOptions::new().read(true).open(file)?;
        fd.read_to_string(&mut raw_str)?;

        if file.ends_with(".yml") || file.ends_with(".yaml") {
            return Self::from_yaml_str(&raw_str);
        }

        if file.ends_with(".json") {
            return Self::from_json_str(&raw_str);
        }

        Err(Error::msg("invalid file format"))
    }

    fn from_yaml_str(s: &str) -> Result<Self> {
        serde_yaml::from_str(s).map_err(|err| Error::msg(format!("fail to load YAML config: {}", err)))
    }

    fn from_json_str(s: &str) -> Result<Self> {
        serde_json::from_str(s).map_err(|err| Error::msg(format!("fail to load JSON config: {}", err)))
    }

    /// Return local service type
    pub fn service_type(&self) -> ServiceType {
        if !self.server && self.client {
            return ServiceType::Client;
        }
        if self.server && !self.client {
            return ServiceType::Server;
        }
        unreachable!("cannot determine service type");
    }

    /// Return DNS server address, default to 8.8.8.8:53
    pub fn get_dns_server(&self) -> Address {
        self.dns_server
            .clone()
            .unwrap_or_else(|| DEFAULT_DNS_SERVER_ADDRESS.parse().unwrap())
    }
}

pub fn check_options(opts: &Options) -> Result<(), &'static str> {
    if opts.generate_cert {
        return Ok(());
    }

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

    // check --daemonize
    // #[cfg(not(target_family = "unix"))]
    // if opts.daemonize {
    //     log::warn!("--daemonize only works on unix.");
    // }

    // check --key
    if opts.key.is_none() && (opts.server_bind.is_some() || opts.server) {
        return Err("-k or --key must be set.");
    }

    // check --proxy-white-list
    if opts.server && opts.proxy_white_list.is_some() {
        return Err("--proxy-white-list can only be set on client.");
    }
    if opts.proxy_white_list.is_some() && opts.proxy_white_list.as_ref().unwrap().is_empty() {
        return Err("--proxy-white-list is set but empty.");
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

    // check --quic
    if opts.quic {
        if opts.tls_cert.is_none() {
            return Err("--tls-cert must be set when --quic is on.");
        }
        if opts.server && opts.tls_key.is_none() {
            return Err("--tls-key must be set when --quic and --server are on.");
        }
    }

    // check --quic-max-concurrency
    if let Some(n) = opts.quic_max_concurrency {
        if n < 1 {
            return Err("--quic-max-concurrency should not be zero.");
        }
    }

    Ok(())
}

#[derive(Clone, Copy)]
pub enum ServiceType {
    Client,
    Server,
}
