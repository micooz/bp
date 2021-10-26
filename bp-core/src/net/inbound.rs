use crate::{
    config,
    event::*,
    net::{
        address::Address,
        socket::{Socket, SocketType},
    },
    options::{Options, ServiceType},
    protocol::*,
};
use anyhow::{Error, Result};
use bytes::Bytes;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{timeout, Duration};

pub struct InboundResolveResult {
    pub proto: DynProtocol,
}

pub struct Inbound {
    opts: Options,

    socket: Arc<Socket>,

    peer_address: SocketAddr,

    #[allow(dead_code)]
    local_addr: SocketAddr,

    protocol_name: Option<String>,

    is_closed: bool,
}

impl Inbound {
    pub fn new(socket: Socket, opts: Options) -> Self {
        let socket = Arc::new(socket);
        let local_addr = socket.local_addr().unwrap();
        let peer_address = socket.peer_addr().unwrap();

        Self {
            opts,
            socket,
            peer_address,
            local_addr,
            protocol_name: None,
            is_closed: false,
        }
    }

    pub fn socket_type(&self) -> SocketType {
        self.socket.socket_type()
    }

    pub async fn try_resolve(&mut self) -> Result<InboundResolveResult> {
        fn direct(addr: &Address, pending_buf: Option<Bytes>) -> DynProtocol {
            let mut direct = Box::new(Direct::default());

            direct.set_resolved_result(ResolvedResult {
                protocol: ProtocolType::Direct,
                address: addr.clone(),
                pending_buf,
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

            return Ok(InboundResolveResult {
                proto: direct(addr, None),
            });
        }

        // client side resolve
        if self.opts.client {
            // obtain iptables redirect dest address first
            let redirect_dest_addr = self.get_redirected_dest_addr();

            let mut try_list: Vec<DynProtocol> = vec![
                Box::new(Socks::new(Some(self.opts.bind.clone()))),
                Box::new(Http::default()),
                Box::new(Https::default()),
            ];

            if self.socket.is_udp() {
                try_list.push(Box::new(Dns::new(self.opts.get_dns_server())));
            }

            // check one by one
            for mut proto in try_list {
                if self.resolve_dest_addr(&mut proto, true).await.is_ok() {
                    // overwrite port if redirect dest addr exist
                    // because http/https sniffer return an inaccurate port number(80 or 443)
                    if let Some(addr) = redirect_dest_addr {
                        let mut resolved = proto.get_resolved_result().unwrap();
                        resolved.address.set_port(addr.port);
                        proto.set_resolved_result(resolved);
                    }
                    return Ok(InboundResolveResult { proto });
                }
            }

            // fallback to iptables redirect dest address
            if let Some(addr) = redirect_dest_addr.as_ref() {
                log::info!(
                    "[{}] [{}] fallback to use iptables's REDIRECT dest address {}",
                    self.peer_address,
                    self.socket.socket_type(),
                    addr
                );

                return Ok(InboundResolveResult {
                    proto: direct(addr, None),
                });
            }
        }

        // server side resolve
        if self.opts.server {
            let mut proto = init_transport_protocol(&self.opts);
            self.resolve_dest_addr(&mut proto, false).await?;

            let resolved = proto.get_resolved_result().unwrap();
            let buf = resolved.pending_buf.as_ref();

            // check dns packet
            if buf.is_some() && Dns::check_dns_query(&buf.unwrap()[..]) {
                proto.set_resolved_result(ResolvedResult {
                    // rewrite dns server address to --dns-server
                    address: self.opts.get_dns_server(),
                    protocol: ProtocolType::Dns,
                    pending_buf: resolved.pending_buf,
                });
            }

            return Ok(InboundResolveResult { proto });
        }

        Err(Error::msg("cannot detect a protocol from incoming data"))
    }

    pub fn set_protocol_name(&mut self, name: String) {
        self.protocol_name = Some(name);
    }

    pub async fn clear_restore(&self) {
        self.socket.clear_restore().await;
    }

    pub async fn handle_pending_data(&self, buf: Bytes, out_proto: &mut DynProtocol, tx: Sender<Event>) -> Result<()> {
        match self.opts.service_type() {
            ServiceType::Client => {
                self.socket.cache(buf).await;

                match out_proto.client_encode(&self.socket).await {
                    Ok(buf) => {
                        tx.send(Event::ClientEncodeDone(buf)).await?;
                    }
                    Err(err) => {
                        tx.send(Event::InboundError(err)).await?;
                    }
                }
            }
            ServiceType::Server => {
                tx.send(Event::ServerDecodeDone(buf)).await?;
            }
        }
        Ok(())
    }

    pub async fn handle_incoming_data(&self, mut in_proto: DynProtocol, mut out_proto: DynProtocol, tx: Sender<Event>) {
        let service_type = self.opts.service_type();
        let socket = self.socket.clone();

        tokio::spawn(async move {
            loop {
                match service_type {
                    ServiceType::Client => match out_proto.client_encode(&socket).await {
                        Ok(buf) => {
                            let _ = tx.send(Event::ClientEncodeDone(buf)).await;
                        }
                        Err(err) => {
                            let _ = tx.send(Event::InboundError(err)).await;
                            break;
                        }
                    },
                    ServiceType::Server => match in_proto.server_decode(&socket).await {
                        Ok(buf) => {
                            let _ = tx.send(Event::ServerDecodeDone(buf)).await;
                        }
                        Err(err) => {
                            let _ = tx.send(Event::InboundError(err)).await;
                            break;
                        }
                    },
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

    #[cfg(feature = "monitor")]
    pub fn snapshot(&self) -> InboundSnapshot {
        InboundSnapshot {
            peer_addr: self.peer_address,
            local_addr: self.local_addr,
            protocol_name: self.protocol_name.clone(),
        }
    }

    async fn resolve_dest_addr(&self, proto: &mut DynProtocol, is_try: bool) -> Result<()> {
        let peer_address = self.peer_address;
        let socket = self.socket.as_ref();
        let socket_type = socket.socket_type();
        let proto_name = proto.get_name();

        let inner_log = |msg: String| {
            if is_try {
                log::trace!("{}", &msg);
            } else {
                log::info!("{}", &msg);
            }
        };

        inner_log(format!(
            "[{}] [{}] use [{}] to resolve dest address",
            peer_address, socket_type, &proto_name,
        ));

        let future = proto.resolve_dest_addr(socket);
        let result = timeout(Duration::from_secs(config::DEST_ADDR_RESOLVE_TIMEOUT_SECONDS), future);
        let result = result.await?;

        match result {
            Ok(_) => {
                let resolved = proto.get_resolved_result();

                log::info!(
                    "[{}] [{}] [{}] successfully resolved dest address {}",
                    peer_address,
                    socket_type,
                    &proto_name,
                    resolved.unwrap().address,
                );

                Ok(())
            }
            Err(err) => {
                inner_log(format!(
                    "[{}] [{}] use [{}] to resolve dest address failed due to: {}",
                    peer_address, socket_type, &proto_name, err,
                ));
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
