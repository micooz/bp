use bp::{bootstrap, Options};
use clap::Clap;

fn main() {
    env_logger::init();

    let opts: Options = Clap::parse();

    // check -c or -s
    if !opts.client && !opts.server {
        log::error!("-c or -s must be set");
        return;
    }
    if opts.client && opts.server {
        log::error!("-c or -s can only be set one");
        return;
    }

    // check --remote-host and --remote-port
    if opts.client && (opts.remote_host == None || opts.remote_port == None) {
        if opts.remote_host == None {
            log::error!("--remote-host must be set when specify -c");
        }
        if opts.remote_port == None {
            log::error!("--remote-port must be set when specify -c");
        }
        return;
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    runtime.block_on(async {
        bootstrap(opts).await;
    });
}
