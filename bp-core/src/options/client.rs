use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{DEFAULT_CLIENT_SERVICE_ADDRESS, DEFAULT_DNS_SERVER_ADDRESS},
    net::address::Address,
    protos::EncryptionMethod,
    HttpBasicAuth,
};

// The following getters are for serde deserializing

fn get_default_bind() -> Address {
    DEFAULT_CLIENT_SERVICE_ADDRESS.parse().unwrap()
}

fn get_default_encryption() -> EncryptionMethod {
    EncryptionMethod::EncryptRandomPadding
}

fn get_default_dns_server() -> Address {
    DEFAULT_DNS_SERVER_ADDRESS.parse().unwrap()
}

#[derive(clap::Args, Deserialize, Serialize, Debug, Clone)]
pub struct ClientOptions {
    /// Configuration file in YAML/JSON format [default: <empty>]
    #[clap(long)]
    #[serde(skip)]
    pub config: Option<String>,

    /// Local service bind address
    #[clap(short, long, default_value = DEFAULT_CLIENT_SERVICE_ADDRESS)]
    #[serde(default = "get_default_bind")]
    pub bind: Address,

    /// Basic authorization required for HTTP Proxy, e,g. "user:pass" [default: <empty>]
    #[clap(long)]
    pub with_basic_auth: Option<HttpBasicAuth>,

    /// Server bind address. If not set, bp will relay directly [default: <empty>]
    #[clap(long)]
    pub server_bind: Option<Address>,

    /// Start a PAC server at the same time, requires --acl [default: <empty>]
    #[clap(long)]
    pub pac_bind: Option<Address>,

    /// Proxy target used by PAC file, requires --pac-bind [default: --bind]
    #[clap(long)]
    pub pac_proxy: Option<Address>,

    /// Symmetric encryption key, required if --server-bind is set [default: <empty>]
    #[clap(short, long)]
    pub key: Option<String>,

    /// Data encryption method, e.g, "plain" or "erp"
    #[clap(short, long, default_value = "erp")]
    #[serde(default = "get_default_encryption")]
    pub encryption: EncryptionMethod,

    /// Check ACL before proxy, pass a file path [default: <empty>]
    #[clap(long)]
    pub acl: Option<String>,

    /// Redirect all incoming data to this destination, for testing [default: <empty>]
    #[clap(long)]
    pub pin_dest_addr: Option<Address>,

    /// Convert udp to tcp requires --server-bind to be set if true [default: false]
    #[clap(long)]
    #[serde(default)]
    pub udp_over_tcp: bool,

    /// DNS server address
    #[clap(long, default_value = DEFAULT_DNS_SERVER_ADDRESS)]
    #[serde(default = "get_default_dns_server")]
    pub dns_server: Address,

    /// Enable TLS for Transport Layer [default: false]
    #[clap(long)]
    #[serde(default)]
    pub tls: bool,

    /// Enable QUIC for Transport Layer [default: false]
    #[clap(long)]
    #[serde(default)]
    pub quic: bool,

    /// The max number of QUIC connections [default: Infinite]
    #[clap(long)]
    pub quic_max_concurrency: Option<u16>,

    /// Certificate for QUIC or TLS [default: <empty>]
    #[clap(long)]
    pub tls_cert: Option<String>,

    /// Enable monitor push service [default: <empty>]
    #[clap(long)]
    pub monitor: Option<Address>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            config: None,
            bind: get_default_bind(),
            with_basic_auth: None,
            server_bind: None,
            pac_bind: None,
            pac_proxy: None,
            key: None,
            encryption: get_default_encryption(),
            acl: None,
            pin_dest_addr: None,
            udp_over_tcp: false,
            dns_server: get_default_dns_server(),
            tls: false,
            quic: false,
            quic_max_concurrency: None,
            tls_cert: None,
            monitor: None,
        }
    }
}

impl ClientOptions {
    pub fn check(&self) -> Result<()> {
        if self.server_bind.is_none() {
            log::warn!("--server-bind is not set, bp will relay directly.");
        }

        if self.server_bind.is_some() && self.key.is_none() {
            return Err(Error::msg("-k or --key must be set."));
        }

        if self.pac_bind.is_some() && self.acl.is_none() {
            return Err(Error::msg("--pac-bind requires --acl to be set."));
        }

        if self.pac_proxy.is_some() && self.pac_bind.is_none() {
            return Err(Error::msg("--pac-proxy requires --pac-bind to be set."));
        }

        if self.udp_over_tcp && self.server_bind.is_none() {
            return Err(Error::msg("--udp-over-tcp requires --server-bind to be set."));
        }

        if self.tls && self.quic {
            return Err(Error::msg("--tls and --quic can only set one."));
        }

        if (self.tls || self.quic) && self.tls_cert.is_none() {
            return Err(Error::msg("--tls-cert must be set when --tls or --quic is on."));
        }

        // check --quic-max-concurrency
        if let Some(n) = self.quic_max_concurrency {
            if n < 1 {
                return Err(Error::msg("--quic-max-concurrency should not be zero."));
            }
        }

        Ok(())
    }
}
