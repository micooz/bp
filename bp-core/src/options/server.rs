use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{DEFAULT_DNS_SERVER_ADDRESS, DEFAULT_SERVER_SERVICE_ADDRESS},
    net::address::Address,
    protos::EncryptionMethod,
};

// The following getters are for serde deserializing

fn get_default_bind() -> Address {
    DEFAULT_SERVER_SERVICE_ADDRESS.parse().unwrap()
}

fn get_default_encryption() -> EncryptionMethod {
    EncryptionMethod::EncryptRandomPadding
}

fn get_default_dns_server() -> Address {
    DEFAULT_DNS_SERVER_ADDRESS.parse().unwrap()
}

#[derive(clap::Args, Deserialize, Serialize, Debug, Clone)]
pub struct ServerOptions {
    /// Configuration file in YAML/JSON format [default: <empty>]
    #[clap(long)]
    #[serde(skip)]
    pub config: Option<String>,

    /// Local service bind address
    #[clap(short, long, default_value = DEFAULT_SERVER_SERVICE_ADDRESS)]
    #[serde(default = "get_default_bind")]
    pub bind: Address,

    /// Symmetric encryption key
    #[clap(short, long)]
    pub key: Option<String>,

    /// Data encryption method, e.g, "plain" or "erp"
    #[clap(short, long, default_value = "erp")]
    #[serde(default = "get_default_encryption")]
    pub encryption: EncryptionMethod,

    /// Check ACL before proxy, pass a file path [default: <empty>]
    #[clap(long)]
    pub acl: Option<String>,

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

    /// Certificate file for QUIC or TLS [default: <empty>]
    #[clap(long)]
    pub tls_cert: Option<String>,

    /// Private key file for QUIC or TLS [default: <empty>]
    #[clap(long)]
    pub tls_key: Option<String>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            config: None,
            bind: get_default_bind(),
            key: None,
            encryption: get_default_encryption(),
            acl: None,
            dns_server: get_default_dns_server(),
            tls: false,
            quic: false,
            tls_cert: None,
            tls_key: None,
        }
    }
}

impl ServerOptions {
    pub fn check(&self) -> Result<()> {
        if self.key.is_none() {
            return Err(Error::msg("--key must be set."));
        }

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
