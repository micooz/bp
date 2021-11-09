use std::process::exit;

use bp_cli::{bootstrap::bootstrap, logging};
use bp_core::{check_options, Options, StartupInfo};
use clap::Parser;
use tokio::sync::oneshot;

#[cfg(feature = "profile")]
#[global_allocator]
static ALLOCATOR: dhat::DhatAlloc = dhat::DhatAlloc;

#[tokio::main]
async fn main() {
    #[cfg(feature = "profile")]
    dhat::Dhat::start_heap_profiling();

    #[cfg(feature = "logging")]
    logging::init();

    let mut opts: Options = Parser::parse();

    // load config from file
    if let Some(config) = opts.config {
        opts = Options::from_file(&config).unwrap_or_else(|err| {
            log::error!("Unrecognized format of {}: {}", &config, err);
            exit(ExitError::ArgumentsError.into());
        });
    }

    match check_options(&opts) {
        Ok(_) => {
            let (tx, _rx) = oneshot::channel::<StartupInfo>();

            if let Err(err) = bootstrap(opts, tx).await {
                log::error!("{}", err);
            }
        }
        Err(err) => {
            log::error!("{}", err);
            exit(ExitError::ArgumentsError.into());
        }
    }
}

enum ExitError {
    ArgumentsError,
}

impl From<ExitError> for i32 {
    fn from(v: ExitError) -> Self {
        match v {
            ExitError::ArgumentsError => -1,
        }
    }
}
