use crate::bootstrap::bootstrap;
use crate::options::{check_options, Options};
use crate::ServiceContext;
use std::sync::Mutex;
use tokio::sync::oneshot;

lazy_static::lazy_static! {
    static ref INCREMENTAL_PORT_NUM:Mutex<u16> = Mutex::new(1080);
}

pub async fn run_bp(mut opts: Options) -> ServiceContext {
    let opts = {
        let mut port = INCREMENTAL_PORT_NUM.lock().unwrap();
        *port += 1;
        opts.bind = format!("{}:{}", "127.0.0.1", port);
        opts
    };

    check_options(&opts).unwrap();

    let (tx, rx) = oneshot::channel::<ServiceContext>();

    tokio::spawn(async {
        bootstrap(opts, tx).await.unwrap();
    });

    if let Ok(v) = rx.await {
        return v;
    }

    unreachable!();

    // let addr = bind.clone();
    // let mut split = addr.split(":");

    // let host: String = split.next().unwrap().into();
    // let port: u16 = split.next().unwrap().parse().unwrap();
}
