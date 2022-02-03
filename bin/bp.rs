use std::process;

use bp_cli::{bootstrap::bootstrap, logging};
use bp_core::{utils::tls::TLS, Cli, Command, StartupInfo};
use clap::StructOpt;
use tokio::sync::oneshot;

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

    let cli: Cli = Cli::parse();

    // generate TLS certificate and private key
    if let Command::Generate(opts) = cli.command {
        if opts.hostname.is_none() {
            log::error!("should set --hostname when --certificate is set");
            exit(ExitError::ArgumentsError);
        }

        let hostname = opts.hostname.unwrap();
        let res = TLS::generate_cert_and_key(vec![hostname], "cert.der", "key.der");

        if let Err(err) = res {
            log::error!("failed to generate TLS certificate due to: {}", err);
            exit(ExitError::ArgumentsError);
        }

        return;
    }

    let mut opts = cli.service_options();

    // try load bp service options from --config
    if let Some(config) = opts.config() {
        if let Err(err) = opts.try_load_from_file(&config) {
            log::error!("Unrecognized format of --config: {}", err);
            exit(ExitError::ArgumentsError);
        }
    }

    // check options
    if let Err(err) = opts.check() {
        log::error!("{}", err);
        exit(ExitError::ArgumentsError);
    }

    // bootstrap bp service
    let (tx, _rx) = oneshot::channel::<StartupInfo>();

    if let Err(err) = bootstrap(opts, tx).await {
        log::error!("{}", err);
        exit(ExitError::BootstrapError);
    }

    log::info!("[{}] process exit with code 0", process::id());
}

fn exit(err: ExitError) -> ! {
    process::exit(err.into());
}

enum ExitError {
    ArgumentsError,
    BootstrapError,
}

impl From<ExitError> for i32 {
    fn from(v: ExitError) -> Self {
        match v {
            ExitError::ArgumentsError => 100,
            ExitError::BootstrapError => 200,
        }
    }
}
