use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tokio_rustls::{TlsAcceptor, TlsStream};

use crate::{global::get_tls_server_config, net::socket::Socket, Shutdown};

pub async fn start_tls_service(
    bind_addr: SocketAddr,
    sender: Sender<Option<Socket>>,
    shutdown: Shutdown,
) -> Result<()> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|err| Error::msg(format!(" tls service start failed from {} due to: {}", bind_addr, err)))?;

    log::info!("service running at tls://{}, waiting for connection...", bind_addr);

    tokio::spawn(async move {
        let acceptor = TlsAcceptor::from(Arc::new(get_tls_server_config()));

        loop {
            let accept = tokio::select! {
                v = listener.accept() => v,
                _ = shutdown.recv() => break,
            };

            let acceptor = acceptor.clone();

            if sender.is_closed() {
                break;
            }

            match accept {
                Ok((tcp_stream, _)) => {
                    let tls_stream = tokio::select! {
                        v = acceptor.accept(tcp_stream) => v,
                        _ = shutdown.recv() => break,
                    }
                    .unwrap();

                    sender
                        .send(Some(Socket::from_tls_stream(TlsStream::Server(tls_stream))))
                        .await
                        .unwrap();
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
