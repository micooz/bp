use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Error, Result};
use bytes::Bytes;
use tokio::{
    sync::mpsc::Sender,
    time::{timeout, Duration},
};

use super::socket::SocketType;
use crate::{
    constants,
    event::*,
    net::{address::Address, socket::Socket},
    proto::*,
    Options, ServiceType,
};

pub struct InboundResolveResult {
    pub proto: DynProtocol,
}

pub struct Inbound {
    opts: Options,

    socket: Arc<Socket>,

    peer_address: SocketAddr,

    protocol_name: Option<String>,

    is_closed: Arc<AtomicBool>,
}

impl Inbound {
    pub fn new(socket: Socket, opts: Options) -> Self {
        let socket = Arc::new(socket);
        let peer_address = socket.peer_addr();

        Self {
            opts,
            socket,
            peer_address,
            protocol_name: None,
            is_closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn socket_type(&self) -> SocketType {
        self.socket.socket_type()
    }

    pub async fn resolve(&mut self) -> Result<InboundResolveResult> {
        let res = self.try_resolve().await?;
        self.socket.disable_restore();
        Ok(res)
    }

    async fn try_resolve(&mut self) -> Result<InboundResolveResult> {
        fn direct(addr: &Address) -> DynProtocol {
            let mut direct = Box::new(Direct::default());

            direct.set_resolved_result(ResolvedResult {
                protocol: ProtocolType::Direct,
                address: addr.clone(),
                pending_buf: None,
            });

            direct
        }

        // client side resolve
        if self.opts.is_client() {
            // check --force-dest-addr flag
            if let Some(addr) = &self.opts.force_dest_addr() {
                log::warn!(
                    "[{}] [{}] --force-dest-addr set, will relay to the fixed dest address {}",
                    self.peer_address,
                    self.socket.socket_type(),
                    addr,
                );
                return Ok(InboundResolveResult { proto: direct(addr) });
            }

            // obtain iptables redirect dest address first
            #[cfg(target_os = "linux")]
            let redirect_dest_addr = self.get_redirected_dest_addr();

            #[cfg(not(target_os = "linux"))]
            let redirect_dest_addr: Option<Address> = None;

            let mut try_list: Vec<DynProtocol> = vec![
                Box::new(Socks::new(Some(self.opts.bind()))),
                Box::new(Http::default()),
                Box::new(Https::default()),
            ];

            if self.socket.is_udp() {
                try_list.push(Box::new(Dns::new(self.opts.dns_server())));
            }

            // check one by one
            for mut proto in try_list {
                if self.resolve_dest_addr(&mut proto, true).await.is_ok() {
                    // overwrite port if redirect_dest_addr exist and it's port is not bp itself.
                    // because http/https sniffer return an inaccurate port number(80 or 443)
                    if let Some(addr) = redirect_dest_addr {
                        if addr.port() != self.opts.bind().port() {
                            let mut resolved = proto.get_resolved_result().unwrap().clone();
                            resolved.set_port(addr.port());
                            proto.set_resolved_result(resolved);
                        }
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

                return Ok(InboundResolveResult { proto: direct(addr) });
            }
        }

        // server side resolve
        if self.opts.is_server() {
            let mut proto = init_protocol(self.opts.protocol(), self.opts.key(), self.opts.service_type());
            self.resolve_dest_addr(&mut proto, false).await?;

            let resolved = proto.get_resolved_result().unwrap().clone();

            // check dns packet
            if let Some(buf) = resolved.pending_buf {
                if Dns::check_dns_query(&buf[..]) {
                    proto.set_resolved_result(ResolvedResult {
                        // rewrite dns server address to --dns-server
                        address: self.opts.dns_server(),
                        protocol: ProtocolType::Dns,
                        pending_buf: Some(buf),
                    });
                }
            }

            return Ok(InboundResolveResult { proto });
        }

        Err(Error::msg("cannot detect a protocol from incoming data"))
    }

    pub fn set_protocol_name(&mut self, name: String) {
        self.protocol_name = Some(name);
    }

    pub async fn handle_pending_data(&self, buf: Bytes, out_proto: &mut DynProtocol, tx: Sender<Event>) -> Result<()> {
        match self.opts.service_type() {
            ServiceType::Client => {
                self.socket.cache(buf);

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

    pub fn handle_incoming_data(&self, mut in_proto: DynProtocol, mut out_proto: DynProtocol, tx: Sender<Event>) {
        let service_type = self.opts.service_type();
        let socket = self.socket.clone();
        let is_closed = self.is_closed.clone();

        tokio::spawn(async move {
            loop {
                if is_closed.load(Ordering::Relaxed) {
                    break;
                }

                // protocol process
                let res = match service_type {
                    ServiceType::Client => out_proto.client_encode(&socket).await,
                    ServiceType::Server => in_proto.server_decode(&socket).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::InboundError(err)).await;
                    break;
                }

                let buf = res.unwrap();

                // send data out
                let event = match service_type {
                    ServiceType::Client => Event::ClientEncodeDone(buf),
                    ServiceType::Server => Event::ServerDecodeDone(buf),
                };

                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
    }

    pub async fn send(&self, buf: bytes::Bytes) -> tokio::io::Result<()> {
        self.socket.send(&buf).await
    }

    pub async fn close(&mut self) -> Result<()> {
        if !self.is_closed.load(Ordering::Relaxed) {
            self.socket.close().await?;
        }
        self.is_closed.store(true, Ordering::Relaxed);

        Ok(())
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
        let result = timeout(
            Duration::from_secs(constants::DEST_ADDR_RESOLVE_TIMEOUT_SECONDS),
            future,
        );
        let result = result.await?;

        match result {
            Ok(_) => {
                let resolved = proto.get_resolved_result().unwrap();

                log::info!(
                    "[{}] [{}] [{}] successfully resolved dest address {}",
                    peer_address,
                    socket_type,
                    &proto_name,
                    resolved.address,
                );

                // http & https request should restore buffer
                if matches!(resolved.protocol, ProtocolType::Http | ProtocolType::Https) {
                    socket.restore();
                }

                Ok(())
            }
            Err(err) => {
                inner_log(format!(
                    "[{}] [{}] use [{}] to resolve dest address failed due to: {}",
                    peer_address, socket_type, &proto_name, err,
                ));
                socket.restore();

                Err(err)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn get_redirected_dest_addr(&self) -> Option<Address> {
        // TODO: get_original_destination_addr always return a value on linux
        use std::os::unix::io::AsRawFd;

        use crate::net::linux::get_original_destination_addr;

        if self.socket.is_udp() {
            return None;
        }

        let fd = self.socket.as_raw_fd();
        let local_addr = self.socket.local_addr()?;

        match get_original_destination_addr(local_addr, fd) {
            Ok(addr) => Some(addr.into()),
            Err(_) => None,
        }
    }
}
