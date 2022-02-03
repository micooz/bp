use anyhow::{Error, Result};

use super::common::OptionsChecker;
use crate::{config::DEFAULT_SERVICE_ADDRESS, net::address::Address, proto::ApplicationProtocol};

#[derive(clap::Args, serde::Deserialize, Default, Clone)]
pub struct ServerOptions {
    /// Configuration file in YAML/JSON format, [default: <empty>]
    #[clap(long)]
    #[serde(skip)]
    pub config: Option<String>,

    /// Local service bind address
    #[clap(short, long, default_value = DEFAULT_SERVICE_ADDRESS)]
    pub bind: Address,

    /// Symmetric encryption key, required
    #[clap(short, long)]
    pub key: String,

    /// Protocol for Application Layer, e.g, "plain" or "erp"
    #[clap(short, long, default_value = "erp")]
    #[serde(default)]
    pub protocol: ApplicationProtocol,

    /// DNS server address [default: 8.8.8.8:53]
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

    /// Certificate file for QUIC or TLS, [default: <empty>]
    #[clap(long)]
    pub tls_cert: Option<String>,

    /// Private key file for QUIC or TLS, [default: <empty>]
    #[clap(long)]
    pub tls_key: Option<String>,
}

impl OptionsChecker for ServerOptions {
    fn check(&self) -> Result<()> {
        if self.tls && self.quic {
            return Err(Error::msg("--tls and --quic can only set one."));
        }

        if self.tls || self.quic {
            if self.tls_cert.is_none() {
                return Err(Error::msg("--tls-cert must be set when --tls or --quic is on."));
            }
            if self.tls_key.is_none() {
                return Err(Error::msg("--tls-key must be set when --tls or --quic is on."));
            }
        }

        Ok(())
    }
}
