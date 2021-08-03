use crate::{
    net::{bootstrap, AcceptResult, Connection},
    options::Options,
    Result,
};
use tokio::sync::mpsc;

pub async fn boot(opts: Options) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<AcceptResult>(32);
    let local_addr = opts.get_local_addr()?;

    // start local service
    tokio::spawn(async move {
        if let Err(err) = bootstrap(local_addr, tx).await {
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
            }

            log::info!("[{}] disconnected", addr);
        });
    }

    Ok(())
}
