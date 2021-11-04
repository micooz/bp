use std::{net::SocketAddr, sync::Arc};

use anyhow::{Error, Result};
use bytes::Bytes;
use tokio::{
    net::TcpSocket,
    sync::mpsc::Sender,
    time::{timeout, Duration},
};

use crate::{
    config,
    event::Event,
    global,
    net::{
        address::{Address, Host},
        socket::{Socket, SocketType},
    },
    options::{Options, ServiceType},
    protocol::{DynProtocol, ResolvedResult},
};

pub struct Outbound {
    opts: Options,

    socket: Option<Arc<Socket>>,

    socket_type: Option<SocketType>,

    peer_address: SocketAddr,

    remote_addr: Option<Address>,

    protocol_name: Option<String>,

    is_closed: bool,

    is_allow_proxy: bool,
}

impl Outbound {
    pub fn new(peer_address: SocketAddr, opts: Options) -> Self {
        Self {
            opts,
            socket: None,
            socket_type: None,
            peer_address,
            remote_addr: None,
            protocol_name: None,
            is_closed: false,
            is_allow_proxy: true,
        }
    }

    pub fn set_socket_type(&mut self, socket_type: SocketType) {
        self.socket_type = Some(socket_type);
    }

    pub fn set_allow_proxy(&mut self, allow: bool) {
        self.is_allow_proxy = allow;
    }

    pub async fn start_connect(&mut self, protocol_name: &str, resolved: &ResolvedResult) -> Result<()> {
        let socket_type = self.socket_type.as_ref().unwrap();
        let peer_address = self.peer_address;

        log::info!(
            "[{}] [{}] use [{}] protocol for outbound",
            peer_address,
            socket_type,
            protocol_name
        );

        let remote_addr = self.get_remote_addr(resolved);

        self.protocol_name = Some(protocol_name.to_string());
        self.remote_addr = Some(remote_addr.clone());

        // resolve dest ip address
        let remote_ip_addr = self.dns_resolve(&remote_addr).await.map_err(|err| {
            let msg = format!(
                "[{}] [{}] resolve ip address of {} failed due to: {}",
                peer_address, socket_type, remote_addr, err
            );
            log::error!("{}", msg);
            Error::msg(msg)
        })?;

        if remote_addr.is_hostname() {
            log::info!(
                "[{}] [{}] connecting to {} resolved to {}...",
                peer_address,
                socket_type,
                remote_addr,
                remote_ip_addr
            );
        } else {
            log::info!("[{}] [{}] connecting to {}...", peer_address, socket_type, remote_addr,);
        }

        // make connection
        let socket = self.connect(remote_ip_addr).await.map_err(|err| {
            let msg = format!(
                "[{}] [{}] connect to {} failed due to: {}",
                peer_address, socket_type, remote_addr, err
            );
            log::error!("{}", msg);
            Error::msg(msg)
        })?;

        self.socket = Some(socket);

        log::info!("[{}] [{}] connected to {}", peer_address, socket_type, remote_addr);

        Ok(())
    }

    pub async fn handle_incoming_data(&self, mut in_proto: DynProtocol, mut out_proto: DynProtocol, tx: Sender<Event>) {
        let socket_type = self.socket_type.as_ref().unwrap();
        let peer_address = self.peer_address;
        let protocol_name = out_proto.get_name();

        log::info!(
            "[{}] [{}] [{}] start relaying data...",
            peer_address,
            socket_type,
            protocol_name
        );

        let service_type = self.opts.service_type();
        let socket = self.socket.clone().unwrap();

        tokio::spawn(async move {
            loop {
                match service_type {
                    ServiceType::Client => match out_proto.client_decode(&socket).await {
                        Ok(buf) => {
                            let _ = tx.send(Event::ClientDecodeDone(buf)).await;
                        }
                        Err(err) => {
                            let _ = tx.send(Event::OutboundError(err)).await;
                            break;
                        }
                    },
                    ServiceType::Server => match in_proto.server_encode(&socket).await {
                        Ok(buf) => {
                            let _ = tx.send(Event::ServerEncodeDone(buf)).await;
                        }
                        Err(err) => {
                            let _ = tx.send(Event::OutboundError(err)).await;
                            break;
                        }
                    },
                };
            }
        });
    }

    pub async fn send(&self, buf: Bytes) -> tokio::io::Result<()> {
        let peer_address = self.peer_address;
        let socket_type = self.socket_type.as_ref().unwrap();
        let protocol_name = self.protocol_name.as_ref().unwrap();
        let socket = self.socket.as_ref().unwrap();

        if socket.is_udp() {
            log::info!(
                "[{}] [{}] [{}] sent an udp packet: {} bytes",
                peer_address,
                socket_type,
                protocol_name,
                buf.len()
            );
        }

        socket.send(&buf).await
    }

    pub async fn close(&mut self) -> Result<()> {
        if !self.is_closed {
            if let Some(socket) = self.socket.as_ref() {
                socket.close().await?;
            }
        }
        self.is_closed = true;

        Ok(())
    }

    #[cfg(feature = "monitor")]
    pub fn snapshot(&self) -> OutboundSnapshot {
        OutboundSnapshot {
            remote_addr: self.remote_addr.clone(),
            protocol_name: self.protocol_name.clone(),
        }
    }

    fn get_remote_addr(&self, resolved: &ResolvedResult) -> Address {
        if self.opts.server || self.opts.server_bind.is_none() || !self.is_allow_proxy {
            resolved.address.clone()
        } else {
            self.opts.server_bind.clone().unwrap()
        }
    }

    async fn dns_resolve(&self, addr: &Address) -> Result<SocketAddr> {
        if addr.is_ip() {
            return Ok(addr.as_socket_addr());
        }

        let socket_type = self.socket_type.as_ref().unwrap();
        let peer_address = self.peer_address;

        log::trace!("[{}] [{}] resolving {}...", peer_address, socket_type, addr);

        let ip_list = match &addr.host {
            Host::Name(name) => {
                // get pre-init resolver
                let resolver = global::SHARED_DATA.get_dns_resolver();
                let resolver = resolver.read().await;
                let resolver = resolver.as_ref().unwrap();

                // set a timeout
                let response = timeout(
                    Duration::from_secs(config::DNS_RESOLVE_TIMEOUT_SECONDS),
                    resolver.lookup_ip(name.as_str()),
                )
                .await??;

                response
                    .iter()
                    .map(|ip| SocketAddr::new(ip, addr.port))
                    .collect::<Vec<SocketAddr>>()
            }
            _ => vec![],
        };

        log::trace!(
            "[{}] [{}] resolved {} to {:?}",
            peer_address,
            socket_type,
            addr,
            ip_list
        );

        Ok(ip_list[0])
    }

    async fn connect(&self, addr: SocketAddr) -> Result<Arc<Socket>> {
        let socket = match self.socket_type.as_ref().unwrap() {
            SocketType::Tcp => {
                #[cfg(target_os = "linux")]
                use std::os::unix::io::AsRawFd;

                let socket = match addr {
                    SocketAddr::V4(..) => TcpSocket::new_v4()?,
                    SocketAddr::V6(..) => TcpSocket::new_v6()?,
                };

                #[cfg(target_os = "linux")]
                if let Err(err) = self.mark_socket(socket.as_raw_fd(), 0xff) {
                    log::error!("set SO_MARK error due to: {}", err);
                }

                let future = socket.connect(addr);
                let stream = timeout(Duration::from_secs(config::TCP_CONNECT_TIMEOUT_SECONDS), future).await??;

                Arc::new(Socket::from_stream(stream))
            }
            SocketType::Udp => {
                let socket = Socket::bind_udp_random_port(addr).await?;
                // TODO: self.mark_socket(socket.as_raw_fd());

                Arc::new(socket)
            }
        };

        Ok(socket)
    }

    #[cfg(target_os = "linux")]
    fn mark_socket(&self, fd: i32, mark: u8) -> Result<()> {
        let mark: libc::c_uint = mark.into();

        let ret = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_MARK,
                &mark as *const _ as *const _,
                std::mem::size_of_val(&mark) as libc::socklen_t,
            )
        };

        if ret != 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::msg(err.to_string()));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct OutboundSnapshot {
    pub remote_addr: Option<Address>,
    pub protocol_name: Option<String>,
}
