use crate::config;
use crate::net::socket::Socket;
use bytes::Bytes;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net;
use tokio::sync::{mpsc, oneshot};

pub enum Transport {
    Tcp,
    Udp,
}

#[derive(Debug)]
pub struct StartupInfo {
    pub bind_addr: SocketAddr,
}

pub fn start_service(
    name: &'static str,
    bind_addr: SocketAddr,
    enable_udp: bool,
    sender_ready: oneshot::Sender<StartupInfo>,
) -> mpsc::Receiver<Socket> {
    let (sender, receiver) = mpsc::channel::<Socket>(32);

    let tcp_sender = sender.clone();
    let udp_sender = sender;

    tokio::spawn(async move {
        if let Err(err) = bind_tcp(name, bind_addr, tcp_sender, sender_ready).await {
            log::error!("[{}] tcp service start failed due to: {}", name, err);
        }
    });

    if enable_udp {
        tokio::spawn(async move {
            if let Err(err) = bind_udp(name, bind_addr, udp_sender).await {
                log::error!("[{}] udp service start failed due to: {}", name, err);
            }
        });
    }

    receiver
}

async fn bind_tcp(
    name: &'static str,
    addr: SocketAddr,
    sender: mpsc::Sender<Socket>,
    sender_ready: oneshot::Sender<StartupInfo>,
) -> std::io::Result<()> {
    let listener = net::TcpListener::bind(&addr).await?;

    log::info!(
        "[{}] service running at tcp://{}, waiting for connection...",
        name,
        addr
    );

    sender_ready.send(StartupInfo { bind_addr: addr }).unwrap();

    loop {
        let (stream, _) = listener.accept().await?;
        let _res = sender.send(Socket::new_tcp(stream)).await;
    }
}

async fn bind_udp<A>(name: &'static str, addr: A, sender: mpsc::Sender<Socket>) -> std::io::Result<()>
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

        let mut buf = vec![0; config::UDP_MTU];
        let (len, addr) = socket.recv_from(&mut buf).await?;

        if let Some(buf) = buf.get(0..len) {
            let socket = Socket::new_udp(socket, addr);

            socket.cache(Bytes::copy_from_slice(buf)).await;

            let _res = sender.send(socket).await;
        }
    }
}
