use bp_cli::{bootstrap::bootstrap, logging};
use bp_core::{check_options, Options, StartupInfo};
use clap::Parser;
use std::process::exit;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    #[cfg(feature = "logging")]
    let log_path = logging::init();

    let mut opts: Options = Parser::parse();

    // load YAML config if set
    if let Some(config) = opts.config {
        opts = Options::from_yaml_file(&config).unwrap_or_else(|err| {
            log::error!("Invalid YAML format: {}", err);
            exit(ExitError::ArgumentsError.into());
        });
    }

    match check_options(&opts) {
        Ok(_) => {
            log::info!("log files are stored at {}", log_path.to_str().unwrap());

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
