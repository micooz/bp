use bp_cli::{
    commands::{generate, service, test},
    options::cli::{Cli, Command},
};
use bp_core::{logging, Startup};
use clap::StructOpt;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    logging::init().unwrap();

    let cli = Cli::parse();

    match cli.command {
        // $ bp generate [OPTIONS]
        Command::Generate(opts) => {
            generate::run(opts).await;
        }
        // $ bp client/server [OPTIONS]
        Command::Client(_) | Command::Server(_) => {
            let opts = cli.service_options();
            let shutdown = tokio::signal::ctrl_c();
            let (startup_sender, _startup_receiver) = mpsc::channel::<Startup>(1);

            let _ = service::run(opts, startup_sender, shutdown).await;
        }
        // $ bp test [OPTIONS]
        Command::Test(opts) => {
            test::run(opts).await;
        }
    }
}
