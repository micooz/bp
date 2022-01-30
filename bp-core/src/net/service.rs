use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::StreamExt;
use quinn::Endpoint;
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::mpsc::Sender,
};

use crate::{config, global, net::socket::Socket};

#[derive(Debug)]
pub struct StartupInfo {
    pub bind_addr: SocketAddr,
    pub bind_ip: String,
    pub bind_host: String,
    pub bind_port: u16,
}

#[async_trait]
pub trait Service {
    async fn start(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()>;
}

pub struct TcpService;
pub struct UdpService;
pub struct QuicService;

#[async_trait]
impl Service for TcpService {
    async fn start(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()> {
        let listener = TcpListener::bind(addr).await.map_err(|err| {
            Error::msg(format!(
                "[{}] tcp service start failed from {} due to: {}",
                name, addr, err
            ))
        })?;

        log::info!(
            "[{}] service running at tcp://{}, waiting for connection...",
            name,
            addr,
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
    async fn start(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()> {
        let socket = Arc::new(UdpSocket::bind(addr).await.map_err(|err| {
            Error::msg(format!(
                "[{}] udp service start failed from {} due to: {}",
                name, addr, err
            ))
        })?);

        log::info!(
            "[{}] service running at udp://{}, waiting for data packets...",
            name,
            addr,
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
    async fn start(name: &'static str, addr: SocketAddr, sender: Sender<Option<Socket>>) -> Result<()> {
        let (_endpoint, mut incoming) = Endpoint::server(global::get_quinn_server_config(), addr).map_err(|err| {
            Error::msg(format!(
                "[{}] quic service start failed from {} due to: {}",
                name, addr, err
            ))
        })?;

        log::info!(
            "[{}] service running at quic://{}, waiting for connection...",
            name,
            addr,
        );

        tokio::spawn(async move {
            loop {
                let sender = sender.clone();
                if sender.is_closed() {
                    break;
                }

                let conn = incoming.next().await.unwrap().await;

                if let Err(err) = conn {
                    log::error!("[{}] cannot establish quic connection due to: {}", name, err);
                    continue;
                }

                let mut conn = conn.unwrap();
                let conn_id = conn.connection.stable_id();
                let peer_addr = conn.connection.remote_address();

                log::info!("[{}] [{}] established new quic connection", peer_addr, conn_id);

                tokio::spawn(async move {
                    while let Some(stream) = conn.bi_streams.next().await {
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
}
