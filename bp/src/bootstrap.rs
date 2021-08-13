use crate::{logging, options::Options};
use bp_lib::{Connection, ConnectionOptions};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct AcceptResult {
    /// The underlying socket.
    pub socket: TcpStream,

    /// The incoming address.
    pub addr: SocketAddr,
}

pub async fn bootstrap(opts: Options) -> std::io::Result<()> {
    logging::setup().await;

    let (tx, mut rx) = mpsc::channel::<AcceptResult>(32);

    // start local service
    let bind_addr = opts.bind.clone();
    tokio::spawn(async move {
        if let Err(err) = listen(bind_addr, tx).await {
            log::error!("service bootstrap failed due to: {}", err);
        }
    });

    // handle connections
    while let Some(accept) = rx.recv().await {
        let addr = accept.socket.peer_addr()?;
        let opts = opts.clone();

        let conn_opts = ConnectionOptions::new(
            opts.get_service_type().unwrap(),
            opts.protocol,
            opts.key,
            opts.server_host,
            opts.server_port,
        );

        let mut conn = Connection::new(accept.socket, conn_opts);

        tokio::spawn(async move {
            log::info!("[{}] connected", addr);

            if let Err(err) = conn.handle().await {
                log::error!("{}", err);
                let _ = conn.force_close().await;
            }

            log::info!("[{}] disconnected", addr);
        });
    }

    Ok(())
}

pub async fn listen(bind_addr: String, sender: mpsc::Sender<AcceptResult>) -> std::io::Result<()> {
    let listener = TcpListener::bind(&bind_addr).await?;

    log::info!("service running at {}, waiting for connection...", bind_addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        sender.send(AcceptResult { socket, addr }).await.unwrap();
    }
}
