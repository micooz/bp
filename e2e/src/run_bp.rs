use std::{str::FromStr, sync::Mutex};

use bp_cli::commands::client_server::bootstrap;
use bp_core::{Address, Options, StartupInfo};
use tokio::sync::oneshot;

lazy_static::lazy_static! {
    static ref INCREMENTAL_PORT_NUM :Mutex<u16> = Mutex::new(2080);
}

pub async fn run_bp(mut opts: Options, host: Option<&str>) -> StartupInfo {
    let mut port = INCREMENTAL_PORT_NUM.lock().unwrap();
    *port += 1;

    opts.set_bind(Address::from_str(&format!("{}:{}", host.unwrap_or("127.0.0.1"), port)).unwrap());

    // should release mutex guard for port here
    drop(port);

    opts.check().unwrap();

    let (tx, rx) = oneshot::channel::<StartupInfo>();

    tokio::spawn(async {
        bootstrap(opts, tx).await.unwrap();
    });

    rx.await.unwrap()
}
