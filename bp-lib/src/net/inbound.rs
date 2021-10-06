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
}

impl Inbound {
    pub fn new(socket: socket::Socket, opts: InboundOptions) -> Self {
        let socket = Arc::new(socket);

        let peer_address = socket.peer_addr().unwrap();
        let local_addr = socket.local_addr().unwrap();

        Self {
            opts,
            socket,
            peer_address,
            proxy_address: None,
            local_addr,
            protocol_name: None,
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

        log::info!(
            "[{}] [{}] use [{}] to resolve target address",
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

        let resolve_result = time::timeout(
            time::Duration::from_secs(config::PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS),
            in_proto.resolve_proxy_address(&self.socket),
        )
        .await
        .map_err(|err| map_err(err.to_string()))?;

        self.socket.clear_restore().await;

        let resolved = match resolve_result {
            Ok(v) => v,
            Err(err) => match self.get_redirected_dest_addr() {
                Some(addr) => {
                    log::info!(
                        "[{}] [{}] fallback to use iptables's REDIRECT target address {}",
                        self.peer_address,
                        self.socket.socket_type(),
                        addr
                    );
                    protocol::ResolvedResult {
                        protocol: String::from("REDIRECT"),
                        address: addr,
                        pending_buf: None,
                    }
                }
                None => return Err(err),
            },
        };

        let proxy_address = resolved.address;

        self.protocol_name = Some(resolved.protocol);
        self.proxy_address = Some(proxy_address.clone());

        log::info!(
            "[{}] [{}] [{}] resolved target address {}",
            self.peer_address,
            self.socket.socket_type(),
            self.protocol_name.as_ref().unwrap(),
            self.proxy_address.as_ref().unwrap(),
        );

        in_proto.set_proxy_address(proxy_address.clone());
        out_proto.set_proxy_address(proxy_address);

        log::info!(
            "[{}] [{}] [{}] start relaying data...",
            self.peer_address,
            self.socket.socket_type(),
            self.protocol_name.as_ref().unwrap()
        );

        let ret = (in_proto.clone(), out_proto.clone());

        let service_type = self.opts.service_type;

        // handle pending_buf
        if let Some(buf) = resolved.pending_buf {
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
    pub async fn send(&self, buf: bytes::Bytes) -> tokio::io::Result<()> {
        if self.socket.is_udp() {
            log::info!(
                "[{}] [{}] [{}] sent an udp packet: {} bytes",
                self.peer_address,
                self.socket.socket_type(),
                self.protocol_name.as_ref().unwrap(),
                buf.len()
            );
        }

        self.socket.send(&buf).await
    }

    /// close the bound
    pub async fn close(&mut self) -> Result<()> {
        self.socket.close().await?;

        Ok(())
    }

    pub fn snapshot(&self) -> InboundSnapshot {
        InboundSnapshot {
            peer_addr: self.peer_address,
            local_addr: self.local_addr,
            protocol_name: self.protocol_name.clone(),
        }
    }

    fn get_redirected_dest_addr(&self) -> Option<net::Address> {
        let fd = self.socket.get_socket_fd();

        if self.socket.is_udp() || fd.is_none() {
            return None;
        }

        // TODO: get_original_destination_addr always return a value on linux
        #[cfg(target_os = "linux")]
        use crate::net::linux::get_original_destination_addr;

        #[cfg(target_os = "linux")]
        return match get_original_destination_addr(self.local_addr, fd.unwrap()) {
            Ok(addr) => Some(addr.into()),
            Err(_) => None,
        };

        #[cfg(not(target_os = "linux"))]
        None
    }
}

#[derive(Debug)]
pub struct InboundSnapshot {
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub protocol_name: Option<String>,
}
