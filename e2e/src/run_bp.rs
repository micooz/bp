use std::{str::FromStr, sync::Mutex};

use bp_cli::bootstrap::bootstrap;
use bp_core::{check_options, Address, Options, StartupInfo};
use tokio::sync::oneshot;

lazy_static::lazy_static! {
    static ref INCREMENTAL_PORT_NUM :Mutex<u16> = Mutex::new(1080);
}

pub async fn run_bp(mut opts: Options) -> StartupInfo {
    let opts = {
        let mut port = INCREMENTAL_PORT_NUM.lock().unwrap();
        *port += 1;
        opts.bind = Address::from_str(&format!("{}:{}", "localhost", port)).unwrap();
        opts
    };

    check_options(&opts).unwrap();

    let (tx, rx) = oneshot::channel::<StartupInfo>();

    tokio::spawn(async {
        bootstrap(opts, tx).await.unwrap();
    });

    rx.await.unwrap()
}
