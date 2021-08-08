use crate::{
    logging,
    net::{bootstrap, AcceptResult, Connection},
    options::Options,
    Result,
};
use tokio::sync::mpsc;

pub async fn boot(opts: Options) -> Result<()> {
    logging::setup().await;

    let (tx, mut rx) = mpsc::channel::<AcceptResult>(32);
    let bind_addr = opts.bind.clone();

    // start local service
    tokio::spawn(async move {
        if let Err(err) = bootstrap(bind_addr, tx).await {
            log::error!("service bootstrap failed due to: {}", err);
        }
    });

    // handle connections
    while let Some(accept) = rx.recv().await {
        let addr = accept.socket.peer_addr()?;
        let service_type = opts.get_service_type()?;
        let mut conn = Connection::new(accept.socket, opts.clone());

        tokio::spawn(async move {
            log::info!("[{}] connected", addr);

            if let Err(err) = conn.handle(service_type).await {
                log::error!("{}", err);
                let _ = conn.force_close().await;
            }

            log::info!("[{}] disconnected", addr);
        });
    }

    Ok(())
}
