use std::str::FromStr;

use anyhow::{Error, Result};

// use crate::constants::DEFAULT_BIND_ADDRESS;

pub enum RunType {
    Client,
    Server,
}

impl ToString for RunType {
    fn to_string(&self) -> String {
        match self {
            RunType::Client => "CLIENT".to_string(),
            RunType::Server => "SERVER".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CryptoMethod {
    None,
    Base64,
}

impl FromStr for CryptoMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s.to_lowercase().as_str() {
            "base64" => Self::Base64,
            _ => Self::None,
        };
        Ok(v)
    }
}

impl ToString for CryptoMethod {
    fn to_string(&self) -> String {
        match self {
            CryptoMethod::None => "".into(),
            CryptoMethod::Base64 => "base64".into(),
        }
    }
}

impl Default for CryptoMethod {
    fn default() -> Self {
        Self::None
    }
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct WebOptions {
    /// Run as client [default: false]
    #[clap(long, short)]
    pub client: bool,

    /// Run as server [default: false]
    #[clap(long, short)]
    pub server: bool,

    /// Web server launch address [default: 127.0.0.1:8080]
    #[clap(long)]
    pub bind: Option<String>,

    /// Apply encryption/decryption for all response/request [default: '']
    #[clap(long, default_value = "")]
    pub crypto: CryptoMethod,
}

impl WebOptions {
    pub fn check(&self) -> Result<()> {
        if !self.client && !self.server {
            return Err(Error::msg("--client or --server must be set one."));
        }
        if self.client && self.server {
            return Err(Error::msg("--client and --server can only be set one."));
        }
        Ok(())
    }

    pub fn run_type(&self) -> RunType {
        if self.client {
            return RunType::Client;
        }
        if self.server {
            return RunType::Server;
        }
        unreachable!();
    }
}

// impl Default for Options {
//     fn default() -> Self {
//         Self {
//             client: false,
//             server: false,
//             bind: Some(DEFAULT_BIND_ADDRESS.to_string()),
//         }
//     }
// }
