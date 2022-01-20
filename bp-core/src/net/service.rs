use std::sync::Arc;

use anyhow::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use quinn::Endpoint;
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::mpsc::Sender,
};

use crate::{
    config,
    global::GLOBAL_DATA,
    net::{address::Address, socket::Socket},
};

#[derive(Debug)]
pub struct StartupInfo {
    pub bind_addr: Address,
}

#[async_trait]
pub trait Service {
    async fn start(name: &'static str, bind_addr: &Address, sender: Sender<Option<Socket>>) -> Result<()>;
}

pub struct TcpService;
pub struct UdpService;
pub struct QuicService;

#[async_trait]
impl Service for TcpService {
    async fn start(name: &'static str, addr: &Address, sender: Sender<Option<Socket>>) -> Result<()> {
        let listener = TcpListener::bind(addr.to_string())
            .await
            .map_err(|err| Error::msg(format!("[{}] tcp service start failed due to: {}", name, err)))?;

        log::info!(
            "[{}] service running at tcp://{}, waiting for connection...",
            name,
            addr.as_string()
        );

        tokio::spawn(async move {
            loop {
                let accept = listener.accept().await;

                if sender.is_closed() {
                    break;
                }

                match accept {
                    Ok((stream, _)) => {
                        sender.send(Some(Socket::from_stream(stream))).await.unwrap();
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
}

#[async_trait]
impl Service for UdpService {
    async fn start(name: &'static str, addr: &Address, sender: Sender<Option<Socket>>) -> Result<()> {
        let socket = Arc::new(
            UdpSocket::bind(addr.to_string())
                .await
                .map_err(|err| Error::msg(format!("[{}] udp service start failed due to: {}", name, err)))?,
        );

        log::info!(
            "[{}] service running at udp://{}, waiting for data packets...",
            name,
            addr.as_string()
        );

        tokio::spawn(async move {
            loop {
                let socket = socket.clone();
                let mut buf = vec![0; config::UDP_MTU];
                let recv = socket.recv_from(&mut buf).await;

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
                        log::error!("[{}] encountered an error: {}", name, err);
                        sender.send(None).await.unwrap();
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl Service for QuicService {
    async fn start(name: &'static str, addr: &Address, sender: Sender<Option<Socket>>) -> Result<()> {
        let (_endpoint, mut incoming) = Endpoint::server(GLOBAL_DATA.get_quinn_server_config(), addr.as_socket_addr())?;

        log::info!(
            "[{}] service running at quic://{}, waiting for connection...",
            name,
            addr.as_string()
        );

        tokio::spawn(async move {
            loop {
                let conn = incoming.next().await;

                if sender.is_closed() {
                    break;
                }

                match conn.unwrap().await {
                    Ok(conn) => {
                        sender.send(Some(Socket::from_quinn_conn(conn).await)).await.unwrap();
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
}
