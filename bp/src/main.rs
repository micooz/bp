mod bootstrap;
mod logging;
mod options;

use bootstrap::bootstrap;
use clap::Clap;
use options::Options;
use std::process::exit;

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

#[tokio::main]
async fn main() {
    logging::setup().await;

    let opts: Options = Clap::parse();

    if !opts.client && !opts.server {
        log::error!("--client or --server must be set.");
        exit(ExitError::ArgumentsError.into());
    }

    if opts.client && opts.server {
        log::error!("-c or -s can only be set one.");
        exit(ExitError::ArgumentsError.into());
    }

    // check --key
    if opts.key.is_none() && ((opts.server_host != None && opts.server_port != None) || opts.server) {
        log::error!("-k or --key must be set.");
        exit(ExitError::ArgumentsError.into());
    }

    // check --server-host and --server-port
    if opts.client && (opts.server_host == None || opts.server_port == None) {
        log::warn!("--server-host or --server-port not set, bp will relay directly.");
    }

    // check --proxy-list-path
    if opts.server && opts.proxy_list_path.is_some() {
        log::error!("--proxy-list-path can only be used on client");
        exit(ExitError::ArgumentsError.into());
    }

    if let Err(err) = bootstrap(opts).await {
        log::error!("{}", err);
    }
}
