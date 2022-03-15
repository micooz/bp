use bp_core::{ClientOptions, Options, ServerOptions};
use clap::{Parser, Subcommand};

use super::{generate::GenerateOptions, test::TestOptions, web::WebOptions};

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
            _ => unreachable!(),
        }
    }

    pub fn metadata() -> serde_json::Value {
        // use std::any::{Any, TypeId};
        // use clap::CommandFactory;
        use serde_json::json;

        // let mut meta = serde_json::json!({});

        // // meta.as_object_mut()

        // let mut s = Cli::command();
        // s._build_all();

        // for sub in s.get_subcommands() {
        //     for arg in sub.get_arguments() {
        //         arg.type_id() == TypeId::of::<String>();
        //     }
        // }

        json!({})
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

    /// Run testing utils
    Test(TestOptions),

    /// Run web gui
    Web(WebOptions),
}
