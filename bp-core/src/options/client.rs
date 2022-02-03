use anyhow::{Error, Result};

use super::common::OptionsChecker;
use crate::{config::DEFAULT_SERVICE_ADDRESS, net::address::Address, proto::ApplicationProtocol};

#[derive(clap::Args, serde::Deserialize, Default, Clone)]
pub struct ClientOptions {
    /// Configuration file in YAML/JSON format, [default: <empty>]
    #[clap(long)]
    #[serde(skip)]
    pub config: Option<String>,

    /// Local service bind address
    #[clap(short, long, default_value = DEFAULT_SERVICE_ADDRESS)]
    pub bind: Address,

    /// Server bind address. If not set, bp will relay directly, [default: <empty>]
    #[clap(long)]
    pub server_bind: Option<Address>,

    /// Symmetric encryption key, required if --server-bind is set, [default: <empty>]
    #[clap(short, long)]
    pub key: Option<String>,

    /// Protocol for Application Layer, e.g, "plain" or "erp"
    #[clap(short, long, default_value = "erp")]
    #[serde(default)]
    pub protocol: ApplicationProtocol,

    /// Check white list before proxy, pass a file path, [default: <empty>]
    #[clap(long)]
    pub proxy_white_list: Option<String>,

    /// Redirect all incoming data to this destination, for testing, [default: <empty>]
    #[clap(long)]
    pub force_dest_addr: Option<Address>,

    /// Convert udp to tcp requires --server-bind to be set if true [default: false]
    #[clap(long)]
    #[serde(default)]
    pub udp_over_tcp: bool,

    /// DNS server address, [default: 8.8.8.8:53]
    #[clap(long)]
    pub dns_server: Option<Address>,

    /// Enable TLS for Transport Layer, [default: false]
    #[clap(long)]
    #[serde(default)]
    pub tls: bool,

    /// Enable QUIC for Transport Layer, [default: false]
    #[clap(long)]
    #[serde(default)]
    pub quic: bool,

    /// The max number of QUIC connections, [default: 65535]
    #[clap(long)]
    #[serde(default)]
    pub quic_max_concurrency: Option<u16>,

    /// Certificate for QUIC or TLS, [default: <empty>]
    #[clap(long)]
    pub tls_cert: Option<String>,
}

impl OptionsChecker for ClientOptions {
    fn check(&self) -> Result<()> {
        if self.server_bind.is_none() {
            log::warn!("--server-bind is not set, bp will relay directly.");
        }

        if self.server_bind.is_some() && self.key.is_none() {
            return Err(Error::msg("-k or --key must be set."));
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
