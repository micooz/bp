use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use bytes::Bytes;
use tokio::{net::UdpSocket, sync::mpsc::Sender};

use crate::{constants, net::socket::Socket, Shutdown};

pub async fn start_udp_service(
    bind_addr: SocketAddr,
    sender: Sender<Option<Socket>>,
    shutdown: Shutdown,
) -> Result<()> {
    let socket = Arc::new(
        UdpSocket::bind(bind_addr)
            .await
            .map_err(|err| Error::msg(format!("udp service start failed from {} due to: {}", bind_addr, err)))?,
    );

    log::info!("service running at udp://{}, waiting for data packets...", bind_addr);

    tokio::spawn(async move {
        loop {
            let socket = socket.clone();
            let mut buf = vec![0; constants::UDP_MTU];

            let recv = tokio::select! {
                v = socket.recv_from(&mut buf) => v,
                _ =  shutdown.recv() => break,
            };

            if sender.is_closed() {
                break;
            }

            match recv {
                Ok((len, addr)) => {
                    if let Some(buf) = buf.get(0..len) {
                        let socket = Socket::from_udp_socket(socket, addr);
                        socket.cache(Bytes::copy_from_slice(buf));
                        sender.send(Some(socket)).await.unwrap();
                    }
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
