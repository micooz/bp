use bp_cli::{
    commands::{client_server, generate},
    logging,
    options::cli::{Cli, Command},
};
use clap::StructOpt;

#[tokio::main]
async fn main() {
    #[cfg(feature = "profile")]
    // bp_cli::profile::set_prof_active(true);
    tokio::spawn(async {
        use std::time::Duration;
        let _profiler = bp_cli::profile::new_heap();
        tokio::time::sleep(Duration::from_secs(10)).await;
    });

    #[cfg(feature = "logging")]
    logging::init();

    let cli = Cli::parse();

    match cli.command {
        // $ bp generate [OPTIONS]
        Command::Generate(opts) => {
            generate::run(opts).await;
        }
        // $ bp client/server [OPTIONS]
        Command::Client(_) | Command::Server(_) => {
            let opts = cli.service_options();
            client_server::run(opts).await;
        }
    }
}
