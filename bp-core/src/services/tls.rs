use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use tokio::{net::TcpListener, sync::mpsc::Sender};
use tokio_rustls::{TlsAcceptor, TlsStream};

use crate::{global::get_tls_server_config, net::socket::Socket};

pub async fn start_tls_service(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()> {
    let listener = TcpListener::bind(addr).await.map_err(|err| {
        Error::msg(format!(
            "[{}] tls service start failed from {} due to: {}",
            name, addr, err
        ))
    })?;

    log::info!(
        "[{}] service running at tls://{}, waiting for connection...",
        name,
        addr,
    );

    tokio::spawn(async move {
        let acceptor = TlsAcceptor::from(Arc::new(get_tls_server_config()));

        loop {
            let accept = listener.accept().await;
            let acceptor = acceptor.clone();

            if sender.is_closed() {
                break;
            }

            match accept {
                Ok((tcp_stream, _)) => {
                    let tls_stream = acceptor.accept(tcp_stream).await.unwrap();
                    sender
                        .send(Some(Socket::from_tls_stream(TlsStream::Server(tls_stream))))
                        .await
                        .unwrap();
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
