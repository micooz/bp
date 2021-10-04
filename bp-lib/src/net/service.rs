use crate::net::socket;
use std::fmt::Display;
use std::sync::Arc;
use tokio::net;
use tokio::sync::mpsc;

pub enum Transport {
    Tcp,
    Udp,
}

pub fn start_service(name: &'static str, bind_addr: std::net::SocketAddr) -> mpsc::Receiver<socket::Socket> {
    let (sender, receiver) = mpsc::channel::<socket::Socket>(32);

    let tcp_sender = sender.clone();
    let udp_sender = sender;

    tokio::spawn(async move {
        if let Err(err) = bind_tcp(name, bind_addr, tcp_sender).await {
            log::error!("[{}] tcp service start failed due to: {}", name, err);
        }
    });

    tokio::spawn(async move {
        if let Err(err) = bind_udp(name, bind_addr, udp_sender).await {
            log::error!("[{}] udp service start failed due to: {}", name, err);
        }
    });

    receiver
}

async fn bind_tcp(
    name: &'static str,
    addr: std::net::SocketAddr,
    sender: mpsc::Sender<socket::Socket>,
) -> std::io::Result<()> {
    let listener = net::TcpListener::bind(addr).await?;

    log::info!(
        "[{}] service running at tcp://{}, waiting for connection...",
        name,
        addr
    );

    loop {
        let (stream, _) = listener.accept().await?;
        let _res = sender.send(socket::Socket::new_tcp(stream)).await;
    }
}

async fn bind_udp<A>(name: &'static str, addr: A, sender: mpsc::Sender<socket::Socket>) -> std::io::Result<()>
where
    A: net::ToSocketAddrs + Display,
{
    let socket = Arc::new(net::UdpSocket::bind(&addr).await?);

    log::info!(
        "[{}] service running at udp://{}, waiting for data packets...",
        name,
        addr
    );

    loop {
        let socket = socket.clone();

        let mut buf = vec![0; 1500];
        let (len, addr) = socket.recv_from(&mut buf).await?;

        if let Some(buf) = buf.get(0..len) {
            let socket = socket::Socket::new_udp(socket, addr);

            socket.cache(bytes::Bytes::copy_from_slice(buf)).await;

            let _res = sender.send(socket).await;
        }
    }
}
