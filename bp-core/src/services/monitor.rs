use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use bp_monitor::Subscriber;
use tokio::net::UdpSocket;

use crate::{global::monitor_add_subscriber, Shutdown};

pub async fn start_monitor_service(bind_addr: SocketAddr, shutdown: Shutdown) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind(bind_addr).await.map_err(|err| {
        Error::msg(format!(
            "monitor service start failed from {} due to: {}",
            bind_addr, err
        ))
    })?);

    log::info!("service running at udp://{}, waiting for data packets...", bind_addr);

    tokio::spawn(async move {
        loop {
            let socket = socket.clone();
            let mut buf = vec![0; 1];

            let res = tokio::select! {
                v = socket.recv_from(&mut buf) => v,
                _ = shutdown.recv() => break,
            };

            match res {
                Ok((_len, addr)) => {
                    if let Ok(()) = monitor_add_subscriber(Subscriber::Udp((socket, addr))) {
                        log::info!("[{}] added subscriber", addr);
                    }
                }
                Err(err) => {
                    log::error!("encountered an error: {}", err);
                    break;
                }
            }
        }
    });

    Ok(())
}
