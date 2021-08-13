mod bootstrap;
mod logging;
mod options;

use bootstrap::bootstrap;
use clap::Clap;
use options::Options;

fn main() {
    let opts: Options = Clap::parse();

    // check -c or -s
    if !opts.client && !opts.server {
        eprintln!("-c or -s must be set.");
        return;
    }
    if opts.client && opts.server {
        eprintln!("-c or -s can only be set one.");
        return;
    }

    // check --server-host and --server-port
    if opts.client && (opts.server_host == None || opts.server_port == None) {
        println!("[WARN] --server-host or --server-port not set, bp will work as transparent proxy.");
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    runtime.block_on(async {
        if let Err(err) = bootstrap(opts).await {
            eprintln!("bp error: {}", err);
        }
    });
}
