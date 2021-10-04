use crate::{config, event, net, net::socket, protocol, Result, ServiceType};
use bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time;

pub struct InboundOptions {
    pub service_type: ServiceType,
}

pub struct Inbound {
    opts: InboundOptions,

    socket: Arc<socket::Socket>,

    peer_address: SocketAddr,

    proxy_address: Option<net::Address>,

    local_addr: SocketAddr,

    protocol_name: Option<String>,

    /// Whether the bound is closed
    is_closed: bool,
}

impl Inbound {
    pub fn new(socket: socket::Socket, opts: InboundOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let local_addr = socket.local_addr().unwrap();

        #[cfg(not(target_os = "linux"))]
        let proxy_address: Option<net::Address> = None;

        // TODO: get_original_destination_addr always return a value on linux
        #[cfg(target_os = "linux")]
        let proxy_address = {
            use crate::net::linux::get_original_destination_addr;

            if socket.is_udp() {
                None
            } else {
                match get_original_destination_addr(local_addr, socket.get_tcp_socket_fd()) {
                    Ok(addr) => Some(addr.into()),
                    Err(_) => None,
                }
            }
        };

        Self {
            opts,
            socket: Arc::new(socket),
            peer_address,
            proxy_address,
            local_addr,
            protocol_name: None,
            is_closed: false,
        }
    }

    // parse incoming data to get proxy address
    pub async fn resolve_proxy_address(
        &mut self,
        mut in_proto: protocol::DynProtocol,
        mut out_proto: protocol::DynProtocol,
        tx: event::EventSender,
    ) -> Result<(protocol::DynProtocol, protocol::DynProtocol)> {
        self.protocol_name = Some(in_proto.get_name());

        let already_have_proxy_address_before_resolving = self.proxy_address.is_some();

        let (proxy_address, pending_buf) = match self.proxy_address.as_ref() {
            Some(addr) => {
                log::info!(
                    "[{}] [{}] obtained target address [{}] from REDIRECT",
                    self.peer_address,
                    self.socket.socket_type(),
                    addr
                );

                (addr.clone(), None)
            }
            None => {
                log::info!(
                    "[{}] [{}] use [{}] protocol to resolve target address",
                    self.peer_address,
                    self.socket.socket_type(),
                    self.protocol_name.as_ref().unwrap()
                );

                let map_err = |err: String| {
                    format!(
                        "[{}] [{}] [{}] resolve proxy address failed due to: {}",
                        self.peer_address,
                        self.socket.socket_type(),
                        self.protocol_name.as_ref().unwrap(),
                        err
                    )
                };

                let (addr, pending_buf) = time::timeout(
                    time::Duration::from_secs(config::PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS),
                    in_proto.resolve_proxy_address(&self.socket),
                )
                .await
                .map_err(|err| map_err(err.to_string()))?
                .map_err(|err| map_err(err.to_string()))?;

                // set proxy_address
                self.proxy_address = Some(addr.clone());

                log::info!(
                    "[{}] [{}] [{}] resolved target address {}",
                    self.peer_address,
                    self.socket.socket_type(),
                    self.protocol_name.as_ref().unwrap(),
                    self.proxy_address.as_ref().unwrap(),
                );

                (addr, pending_buf)
            }
        };

        in_proto.set_proxy_address(proxy_address.clone());
        out_proto.set_proxy_address(proxy_address);

        if already_have_proxy_address_before_resolving {
            log::info!(
                "[{}] [{}] start relaying data...",
                self.peer_address,
                self.socket.socket_type()
            );
        } else {
            log::info!(
                "[{}] [{}] [{}] start relaying data...",
                self.peer_address,
                self.socket.socket_type(),
                self.protocol_name.as_ref().unwrap()
            );
        }

        let ret = (in_proto.clone(), out_proto.clone());

        let service_type = self.opts.service_type;

        // handle pending_buf
        if let Some(buf) = pending_buf {
            match service_type {
                ServiceType::Client => {
                    self.socket.cache(buf.clone()).await;

                    if let Err(err) = out_proto.client_encode(&self.socket, tx.clone()).await {
                        let _ = tx.send(event::Event::InboundError(err)).await;
                    }
                }
                ServiceType::Server => {
                    let _ = tx.send(event::Event::ServerDecodeDone(buf)).await;
                }
            }
        }

        if self.socket.is_tcp() {
            let socket = self.socket.clone();

            // start receiving data from inbound
            tokio::spawn(async move {
                loop {
                    let res = match service_type {
                        ServiceType::Client => out_proto.client_encode(&socket, tx.clone()).await,
                        ServiceType::Server => in_proto.server_decode(&socket, tx.clone()).await,
                    };

                    if let Err(err) = res {
                        let _ = tx.send(event::Event::InboundError(err)).await;
                        break;
                    }
                }
            });
        }

        Ok(ret)
    }

    /// send data to remote
    pub async fn send(&self, buf: bytes::Bytes) -> Result<()> {
        self.socket.send(&buf).await
    }

    /// close the bound
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await?;
        self.is_closed = true;

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn snapshot(&self) -> InboundSnapshot {
        InboundSnapshot {
            peer_addr: self.peer_address,
            local_addr: self.local_addr,
            protocol_name: self.protocol_name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct InboundSnapshot {
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub protocol_name: Option<String>,
}
