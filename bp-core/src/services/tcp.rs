use std::net::SocketAddr;

use anyhow::{Error, Result};
use tokio::{net::TcpListener, sync::mpsc::Sender};

use crate::net::socket::Socket;

pub async fn start_tcp_service(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()> {
    let listener = TcpListener::bind(addr).await.map_err(|err| {
        Error::msg(format!(
            "[{}] tcp service start failed from {} due to: {}",
            name, addr, err
        ))
    })?;

    log::info!(
        "[{}] service running at tcp://{}, waiting for connection...",
        name,
        addr,
    );

    tokio::spawn(async move {
        loop {
            let accept = listener.accept().await;

            if sender.is_closed() {
                break;
            }

            match accept {
                Ok((stream, _)) => {
                    sender.send(Some(Socket::from_tcp_stream(stream))).await.unwrap();
                }
                Err(err) => {
                    log::error!("[{}] encountered an error: {}", name, err);
                    sender.send(None).await.unwrap();
                    break;
                }
            }
        }
    });

    Ok(())
}
