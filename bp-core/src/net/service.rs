use crate::{
    config,
    net::{address::Address, socket::Socket},
};
use anyhow::{Error, Result};
use bytes::Bytes;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct StartupInfo {
    pub bind_addr: Address,
}

pub async fn start_service(name: &'static str, bind_addr: &Address) -> Result<Receiver<Socket>> {
    let (sender, receiver) = channel::<Socket>(config::SERVICE_CONNECTION_THRESHOLD);

    let sender_tcp = sender.clone();
    let bind_addr_tcp = bind_addr.clone();

    bind_tcp(name, &bind_addr_tcp, sender_tcp)
        .await
        .map_err(|err| Error::msg(format!("[{}] tcp service start failed due to: {}", name, err)))?;

    let sender_udp = sender;
    let bind_addr_udp = bind_addr.clone();

    bind_udp(name, &bind_addr_udp, sender_udp)
        .await
        .map_err(|err| Error::msg(format!("[{}] udp service start failed due to: {}", name, err)))?;

    Ok(receiver)
}

async fn bind_tcp(name: &'static str, addr: &Address, sender: Sender<Socket>) -> Result<()> {
    let listener = TcpListener::bind(addr.to_string()).await?;

    log::info!(
        "[{}] service running at tcp://{}, waiting for connection...",
        name,
        addr.as_string()
    );

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let _res = sender.send(Socket::from_stream(stream)).await;
        }
    });

    Ok(())
}

async fn bind_udp(name: &'static str, addr: &Address, sender: Sender<Socket>) -> Result<()> {
    let socket = Arc::new(UdpSocket::bind(addr.to_string()).await?);

    log::info!(
        "[{}] service running at udp://{}, waiting for data packets...",
        name,
        addr.as_string()
    );

    tokio::spawn(async move {
        loop {
            let socket = socket.clone();

            let mut buf = vec![0; config::UDP_MTU];
            let (len, addr) = socket.recv_from(&mut buf).await.unwrap();

            if let Some(buf) = buf.get(0..len) {
                let socket = Socket::from_udp_socket(socket, addr);

                socket.cache(Bytes::copy_from_slice(buf)).await;

                let _res = sender.send(socket).await;
            }
        }
    });

    Ok(())
}
