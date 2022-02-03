use clap::{Parser, Subcommand};

use super::{client::ClientOptions, common::Options, generate::GenerateOptions, server::ServerOptions};

#[derive(Parser)]
#[clap(name = "bp", version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn service_options(&self) -> Options {
        match &self.command {
            Command::Client(opts) => Options::Client(opts.clone()),
            Command::Server(opts) => Options::Server(opts.clone()),
            Command::Generate(_) => unreachable!(),
        }
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Run bp client
    Client(ClientOptions),

    /// Run bp server
    Server(ServerOptions),

    /// Run file generator
    Generate(GenerateOptions),
}
