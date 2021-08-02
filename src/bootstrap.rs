use crate::{
    net::{bootstrap, AcceptResult, Connection},
    options::Options,
};
use tokio::sync::mpsc;

pub async fn boot(opts: Options) {
    let (tx, mut rx) = mpsc::channel::<AcceptResult>(32);

    let local_addr = opts.get_local_addr();

    // start local service
    tokio::spawn(async move {
        bootstrap(local_addr, tx).await;
    });

    // handle connections
    while let Some(accept) = rx.recv().await {
        let addr = accept.socket.peer_addr().unwrap();
        let mut conn = Connection::new(accept.socket, opts.clone());
        let service_type = opts.get_service_type().unwrap();

        tokio::spawn(async move {
            log::info!("[{}] connected", addr);

            if let Err(err) = conn.handle(service_type).await {
                log::error!("{}", err);
            }

            log::info!("[{}] disconnected", addr);
        });
    }
}
