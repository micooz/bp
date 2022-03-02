use std::net::SocketAddr;

use anyhow::{Error, Result};
use tokio::{net::TcpListener, sync::mpsc::Sender};

use crate::{net::socket::Socket, Shutdown};

pub async fn start_tcp_service(
    bind_addr: SocketAddr,
    sender: Sender<Option<Socket>>,
    shutdown: Shutdown,
) -> Result<()> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|err| Error::msg(format!("tcp service start failed from {} due to: {}", bind_addr, err)))?;

    log::info!("service running at tcp://{}, waiting for connection...", bind_addr);

    tokio::spawn(async move {
        loop {
            let accept = tokio::select! {
                v = listener.accept() => v,
                _ = shutdown.recv() => break,
            };

            if sender.is_closed() {
                break;
            }

            match accept {
                Ok((stream, _)) => {
                    sender.send(Some(Socket::from_tcp_stream(stream))).await.unwrap();
                }
                Err(err) => {
                    log::error!("encountered an error: {}", err);
                    sender.send(None).await.unwrap();
                    break;
                }
            }
        }
    });

    Ok(())
}
