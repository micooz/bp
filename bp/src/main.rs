mod bootstrap;
mod logging;
mod options;

use bootstrap::bootstrap;
use clap::Clap;
use options::Options;

#[tokio::main]
async fn main() {
    logging::setup().await;

    let opts: Options = Clap::parse();

    if !opts.client && !opts.server {
        log::error!("-c or -s must be set.");
        return;
    }
    if opts.client && opts.server {
        log::error!("-c or -s can only be set one.");
        return;
    }

    // check --server-host and --server-port
    if opts.client && (opts.server_host == None || opts.server_port == None) {
        log::warn!("--server-host or --server-port not set, bp will work as transparent proxy.");
    }

    if let Err(err) = bootstrap(opts).await {
        log::error!("{}", err);
    }
}
