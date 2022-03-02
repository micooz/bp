use std::net::SocketAddr;

use anyhow::{Error, Result};
use futures_util::stream::StreamExt;
use quinn::Endpoint;
use tokio::sync::mpsc::Sender;

use crate::{global, Shutdown, Socket};

pub async fn start_quic_service(
    bind_addr: SocketAddr,
    sender: Sender<Option<Socket>>,
    shutdown: Shutdown,
) -> Result<()> {
    let (_endpoint, mut incoming) = Endpoint::server(global::get_quinn_server_config(), bind_addr)
        .map_err(|err| Error::msg(format!("quic service start failed from {} due to: {}", bind_addr, err)))?;

    log::info!("service running at quic://{}, waiting for connection...", bind_addr);

    tokio::spawn(async move {
        loop {
            let shutdown = shutdown.clone();

            let sender = sender.clone();
            if sender.is_closed() {
                break;
            }

            let shutdown_copy = shutdown.clone();
            let conn = tokio::select! {
                v =  incoming.next() => v,
                _ = shutdown_copy.recv() => break,
            };

            let conn = tokio::select! {
                v = conn.unwrap() => v,
                _ = shutdown.recv() => break,
            };

            if let Err(err) = conn {
                log::error!("cannot establish quic connection due to: {}", err);
                continue;
            }

            let mut conn = conn.unwrap();
            let conn_id = conn.connection.stable_id();
            let peer_addr = conn.connection.remote_address();

            log::info!("[{}] [{}] established new quic connection", peer_addr, conn_id);

            tokio::spawn(async move {
                while let Some(stream) = tokio::select! {
                    v = conn.bi_streams.next() => v,
                    _ = shutdown.recv() => None,
                } {
                    match stream {
                        Ok(s) => {
                            log::info!("[{}] [{}] create new quic stream", peer_addr, conn_id);
                            let socket = Socket::from_quic(peer_addr, s);
                            sender.send(Some(socket)).await.unwrap();
                        }
                        Err(err) => {
                            if matches!(err, quinn::ConnectionError::ApplicationClosed { .. }) {
                                log::info!("[{}] [{}] quic stream closed", peer_addr, conn_id);
                            } else {
                                log::warn!("[{}] [{}] quic stream error due to: {}", peer_addr, conn_id, err);
                            }
                            break;
                        }
                    };
                }
            });
        }
    });

    Ok(())
}
