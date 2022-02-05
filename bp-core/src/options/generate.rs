use std::str::FromStr;

use anyhow::{Error, Result};

#[derive(clap::Args, Default)]
pub struct GenerateOptions {
    /// Generate bp configuration file, [default: <empty>]
    #[clap(long)]
    pub config: Option<String>,

    /// Configuration type for --config, e,g. "client" or "server"
    #[clap(long, default_value = "client")]
    pub config_type: ConfigType,

    /// Generate self-signed TLS certificates(in DER format) to CWD, [default: false]
    #[clap(long)]
    pub certificate: bool,

    /// Hostname for generating TLS certificates, [default: <empty>]
    #[clap(long)]
    pub hostname: Option<String>,
}

pub enum ConfigType {
    Client,
    Server,
}

impl Default for ConfigType {
    fn default() -> Self {
        Self::Client
    }
}

impl FromStr for ConfigType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(Self::Client),
            "server" => Ok(Self::Server),
            _ => Err(format!("unrecognized value: {}", s)),
        }
    }
}

impl GenerateOptions {
    pub fn check(&self) -> Result<()> {
        // check --certificate and --hostname
        if self.certificate && self.hostname.is_none() {
            return Err(Error::msg("--hostname is required when --certificate is on"));
        }

        Ok(())
    }
}
