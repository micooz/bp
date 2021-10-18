use crate::net::Address;
use crate::{
    config,
    event::*,
    net::{socket::Socket, ServiceType},
    protocol::*,
    Options, Result,
};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{timeout, Duration};

pub enum InboundResolveResult {
    Proxy(DynProtocol),
    Direct(DynProtocol),
    Deny,
}

pub struct Inbound {
    opts: Options,

    socket: Arc<Socket>,

    peer_address: SocketAddr,

    local_addr: SocketAddr,

    protocol_name: Option<String>,

    is_closed: bool,
}

impl Inbound {
    pub fn new(socket: Socket, opts: Options) -> Self {
        let socket = Arc::new(socket);
        let peer_address = socket.peer_addr().unwrap();
        let local_addr = socket.local_addr().unwrap();

        Self {
            opts,
            socket,
            peer_address,
            local_addr,
            protocol_name: None,
            is_closed: false,
        }
    }

    pub async fn try_resolve(&mut self) -> Result<InboundResolveResult> {
        fn create_direct(addr: &Address) -> DynProtocol {
            let mut direct = Box::new(Direct::default());

            direct.set_resolved_result(ResolvedResult {
                protocol: direct.get_name(),
                address: addr.clone(),
                pending_buf: None,
            });

            direct
        }

        // check --force-dest-addr flag
        if let Some(addr) = &self.opts.force_dest_addr {
            log::warn!(
                "[{}] [{}] --force-dest-addr set, will relay to the fixed dest address {}",
                self.peer_address,
                self.socket.socket_type(),
                addr,
            );

            return Ok(InboundResolveResult::Direct(create_direct(addr)));
        }

        // check DNS queries
        if self.check_dns_query().await? {
            return Ok(InboundResolveResult::Direct(create_direct(&Address::default())));
        }

        // client side resolve
        if self.opts.client {
            let try_list: [DynProtocol; 3] = [
                Box::new(Socks::new(Some(self.opts.bind.clone()))),
                Box::new(Http::default()),
                Box::new(Https::default()),
            ];

            for mut proto in try_list {
                if self.resolve_dest_addr(&mut proto).await.is_ok() {
                    self.protocol_name = Some(proto.get_name());
                    return Ok(InboundResolveResult::Proxy(proto));
                }
            }

            // iptable redirect
            if let Some(addr) = self.get_redirected_dest_addr().as_ref() {
                log::info!(
                    "[{}] [{}] fallback to use iptables's REDIRECT dest address {}",
                    self.peer_address,
                    self.socket.socket_type(),
                    addr
                );

                return Ok(InboundResolveResult::Direct(create_direct(addr)));
            }
        }

        // server side resolve
        if self.opts.server {
            let mut proto = init_transport_protocol(&self.opts);
            self.resolve_dest_addr(&mut proto).await?;

            return Ok(InboundResolveResult::Proxy(proto));
        }

        Ok(InboundResolveResult::Deny)
    }

    pub async fn clear_restore(&self) {
        self.socket.clear_restore().await;
    }

    pub async fn handle_pending_data(&self, buf: Bytes, out_proto: &mut DynProtocol, tx: Sender<Event>) {
        match self.opts.service_type() {
            ServiceType::Client => {
                self.socket.cache(buf).await;

                if let Err(err) = out_proto.client_encode(&self.socket, tx.clone()).await {
                    let _ = tx.send(Event::InboundError(err)).await;
                }
            }
            ServiceType::Server => {
                let _ = tx.send(Event::ServerDecodeDone(buf)).await;
            }
        }
    }

    pub async fn handle_incoming_data(&self, mut in_proto: DynProtocol, mut out_proto: DynProtocol, tx: Sender<Event>) {
        let service_type = self.opts.service_type();
        let socket = self.socket.clone();

        tokio::spawn(async move {
            loop {
                let res = match service_type {
                    ServiceType::Client => out_proto.client_encode(&socket, tx.clone()).await,
                    ServiceType::Server => in_proto.server_decode(&socket, tx.clone()).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::InboundError(err)).await;
                    break;
                }
            }
        });
    }

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

    async fn check_dns_query(&self) -> Result<bool> {
        if self.socket.is_udp() && self.opts.dns_over_tcp {
            let buf = self.socket.read_some().await?;
            self.socket.restore().await;

            let is_dns = Dns::parse(&buf[..]).is_ok();

            if is_dns {
                self.socket.clear_restore().await;
            }

            return Ok(is_dns);
        }

        Ok(false)
    }

    async fn resolve_dest_addr(&self, proto: &mut DynProtocol) -> Result<()> {
        let peer_address = self.peer_address;
        let socket = self.socket.as_ref();
        let socket_type = socket.socket_type();
        let proto_name = proto.get_name();

        log::info!(
            "[{}] [{}] use [{}] to resolve dest address",
            peer_address,
            socket_type,
            &proto_name,
        );

        let future = proto.resolve_dest_addr(socket);
        let result = timeout(Duration::from_secs(config::DEST_ADDR_RESOLVE_TIMEOUT_SECONDS), future);
        let result = result.await?;

        match result {
            Ok(_) => {
                let resolved = proto.get_resolved_result();

                log::info!(
                    "[{}] [{}] [{}] resolved dest address {}",
                    peer_address,
                    socket_type,
                    &proto_name,
                    resolved.unwrap().address,
                );

                Ok(())
            }
            Err(err) => {
                log::info!(
                    "[{}] [{}] use [{}] to resolve dest address failed due to: {}",
                    peer_address,
                    socket_type,
                    &proto_name,
                    err,
                );
                socket.restore().await;

                Err(err)
            }
        }
    }

    fn get_redirected_dest_addr(&self) -> Option<Address> {
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
