use anyhow::{Error, Result};
use clap::Parser;

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

#[derive(Parser, Debug, Clone)]
#[clap(name = "bp-web", version, about)]
pub struct Options {
    /// Run as client [default: false]
    #[clap(long, short)]
    pub client: bool,

    /// Run as server [default: false]
    #[clap(long, short)]
    pub server: bool,

    /// Web server launch address [default: 127.0.0.1:8080]
    #[clap(long)]
    pub bind: Option<String>,
}

impl Options {
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
