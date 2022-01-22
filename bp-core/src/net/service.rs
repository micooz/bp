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
    net::{address::Address, dns::dns_resolve, socket::Socket},
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
        let ip = if addr.is_hostname() {
            dns_resolve(addr).await?
        } else {
            addr.as_socket_addr()
        };

        let (_endpoint, mut incoming) = Endpoint::server(GLOBAL_DATA.get_quinn_server_config(), ip)?;

        log::info!(
            "[{}] service running at quic://{}, waiting for connection...",
            name,
            addr.as_string()
        );

        tokio::spawn(async move {
            loop {
                let sender = sender.clone();
                if sender.is_closed() {
                    break;
                }

                let conn = incoming.next().await.unwrap().await;

                if let Err(err) = conn {
                    log::error!("[{}] encountered an error: {}", name, err);
                    sender.send(None).await.unwrap();
                    break;
                }

                let mut conn = conn.unwrap();
                let conn_id = conn.connection.stable_id();
                let peer_addr = conn.connection.remote_address();

                log::info!("[{}] [{}] established new quic connection", peer_addr, conn_id);

                tokio::spawn(async move {
                    while let Some(stream) = conn.bi_streams.next().await {
                        match stream {
                            Ok(s) => {
                                log::info!("[{}] [{}] handle new quic stream", peer_addr, conn_id);
                                let socket = Socket::from_quic(peer_addr, s);
                                sender.send(Some(socket)).await.unwrap();
                            }
                            Err(err) => {
                                if matches!(err, quinn::ConnectionError::ApplicationClosed { .. }) {
                                    log::info!("[{}] [{}] quic stream closed", peer_addr, conn_id);
                                } else {
                                    log::error!("[{}] [{}] quic stream error due to: {}", peer_addr, conn_id, err);
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
}
