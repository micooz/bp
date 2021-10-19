use bp_cli::{bootstrap::bootstrap, logging};
use bp_core::{check_options, Options, StartupInfo};
use clap::Parser;
use std::process::exit;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    #[cfg(feature = "logging")]
    logging::init().await;

    let opts: Options = Parser::parse();

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
