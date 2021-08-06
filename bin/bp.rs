use bp::{bootstrap, Options};
use clap::Clap;

fn main() {
    let opts: Options = Clap::parse();

    // check -c or -s
    if !opts.client && !opts.server {
        eprintln!("-c or -s must be set");
        return;
    }
    if opts.client && opts.server {
        eprintln!("-c or -s can only be set one");
        return;
    }

    // check --remote-host and --remote-port
    if opts.client && (opts.remote_host == None || opts.remote_port == None) {
        if opts.remote_host == None {
            eprintln!("--remote-host must be set when specify -c");
        }
        if opts.remote_port == None {
            eprintln!("--remote-port must be set when specify -c");
        }
        return;
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    runtime.block_on(async {
        if let Err(err) = bootstrap(opts).await {
            log::error!("bp error: {}", err);
        }
    });
}
