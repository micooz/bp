use crate::{config, event::*, global, net, net::socket, protocol::*, Options, Result};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time;

type Proto = DynProtocol;

pub struct InboundResolveResult {
    pub in_proto: Proto,

    pub out_proto: Proto,

    pub is_proxy: bool,
}

pub struct Inbound {
    opts: Options,

    socket: Arc<socket::Socket>,

    peer_address: SocketAddr,

    proxy_address: Option<net::Address>,

    local_addr: SocketAddr,

    protocol_name: Option<String>,

    is_closed: bool,
}

impl Inbound {
    pub fn new(socket: socket::Socket, opts: Options) -> Self {
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
            is_closed: false,
        }
    }

    // parse incoming data to get proxy address
    pub async fn resolve_addr(&mut self, mut proto: Proto, tx: EventSender) -> Result<InboundResolveResult> {
        self.protocol_name = Some(proto.get_name());

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

        let resolved = if let Some(addr) = &self.opts.force_dest_addr {
            log::warn!(
                "[{}] [{}] --force-dest-addr set, will relay to the fixed target address {}",
                self.peer_address,
                self.socket.socket_type(),
                addr,
            );

            ResolvedResult {
                protocol: String::from("none"),
                address: addr.clone(),
                pending_buf: None,
            }
        } else {
            let resolve_result = time::timeout(
                time::Duration::from_secs(config::PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS),
                proto.resolve_proxy_address(&self.socket),
            )
            .await
            .map_err(|err| map_err(err.to_string()))?;

            match resolve_result {
                Ok(v) => v,
                Err(err) => match self.get_redirected_dest_addr() {
                    Some(addr) => {
                        log::info!(
                            "[{}] [{}] fallback to use iptables's REDIRECT target address {}",
                            self.peer_address,
                            self.socket.socket_type(),
                            addr
                        );
                        ResolvedResult {
                            protocol: String::from("REDIRECT"),
                            address: addr,
                            pending_buf: None,
                        }
                    }
                    None => return Err(err),
                },
            }
        };

        self.socket.clear_restore().await;

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

        let (mut out_proto, is_proxy) = self.create_outbound_protocol().await;

        proto.set_proxy_address(proxy_address.clone());
        out_proto.set_proxy_address(proxy_address);

        log::info!(
            "[{}] [{}] [{}] start relaying data...",
            self.peer_address,
            self.socket.socket_type(),
            self.protocol_name.as_ref().unwrap()
        );

        let ret = InboundResolveResult {
            in_proto: proto.clone(),
            out_proto: out_proto.clone(),
            is_proxy,
        };

        // handle pending_buf
        if let Some(buf) = resolved.pending_buf {
            self.handle_pending_data(buf, &mut out_proto, tx.clone()).await;
        }

        // start receiving data from inbound
        if self.socket.is_tcp() {
            self.handle_incoming_data(proto, out_proto, tx).await;
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
        if !self.is_closed {
            self.socket.close().await?;
        }
        self.is_closed = true;

        Ok(())
    }

    pub fn snapshot(&self) -> InboundSnapshot {
        InboundSnapshot {
            peer_addr: self.peer_address,
            local_addr: self.local_addr,
            protocol_name: self.protocol_name.clone(),
        }
    }

    async fn create_outbound_protocol(&self) -> (Proto, bool) {
        let service_type = self.opts.service_type();
        let direct = Box::new(Direct::default());

        // server address not provided on client
        if service_type.is_client() && self.opts.server_bind.is_none() {
            return (direct, false);
        }

        // check acl
        let proxy_addr = self.proxy_address.as_ref().unwrap();
        let proxy_addr_host = proxy_addr.host.to_string();

        // white list
        if service_type.is_client() && self.opts.proxy_list_path.is_some() {
            let acl = global::SHARED_DATA.get_acl();

            if !acl.is_host_hit(&proxy_addr_host) {
                log::info!(
                    "[{}] [{}] [{}] is not matched in white list, will use [direct] protocol for outbound",
                    self.peer_address,
                    self.socket.socket_type(),
                    proxy_addr_host,
                );
                return (direct, false);
            }

            log::info!(
                "[{}] [{}] [{}] is matched in white list",
                self.peer_address,
                self.socket.socket_type(),
                proxy_addr_host,
            );
        }

        let proto: Proto = match self.opts.protocol {
            TransportProtocol::Plain => Box::new(Plain::default()),
            TransportProtocol::EncryptRandomPadding => Box::new(Erp::new(self.opts.key.clone().unwrap(), service_type)),
        };

        (proto, true)
    }

    async fn handle_pending_data(&self, buf: Bytes, out_proto: &mut Proto, tx: Sender<Event>) {
        match self.opts.service_type() {
            net::ServiceType::Client => {
                self.socket.cache(buf).await;

                if let Err(err) = out_proto.client_encode(&self.socket, tx.clone()).await {
                    let _ = tx.send(Event::InboundError(err)).await;
                }
            }
            net::ServiceType::Server => {
                let _ = tx.send(Event::ServerDecodeDone(buf)).await;
            }
        }
    }

    async fn handle_incoming_data(&self, mut in_proto: Proto, mut out_proto: Proto, tx: Sender<Event>) {
        let service_type = self.opts.service_type();
        let socket = self.socket.clone();

        tokio::spawn(async move {
            loop {
                let res = match service_type {
                    net::ServiceType::Client => out_proto.client_encode(&socket, tx.clone()).await,
                    net::ServiceType::Server => in_proto.server_decode(&socket, tx.clone()).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::InboundError(err)).await;
                    break;
                }
            }
        });
    }

    fn get_redirected_dest_addr(&self) -> Option<net::Address> {
        #[cfg(target_os = "linux")]
        {
            // TODO: get_original_destination_addr always return a value on linux
            use crate::net::linux::get_original_destination_addr;
            use std::os::unix::io::AsRawFd;

            if self.socket.is_udp() {
                return None;
            }

            let fd = self.socket.as_raw_fd();

            match get_original_destination_addr(self.local_addr, fd) {
                Ok(addr) => Some(addr.into()),
                Err(_) => None,
            }
        }

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
