use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Error, Result};
use bytes::Bytes;
use rustls;
use tokio::{
    net::TcpSocket,
    sync::mpsc::Sender,
    time::{timeout, Duration},
};
use tokio_rustls::{TlsConnector, TlsStream};

use super::socket::SocketType;
use crate::{
    constants,
    event::Event,
    global::{self, get_tls_client_config},
    net::{address::Address, dns::dns_resolve, quic::RandomEndpoint, socket::Socket},
    protos::{DynProtocol, ResolvedResult},
    Options, ServiceType,
};

pub struct Outbound {
    opts: Options,

    socket: Option<Arc<Socket>>,

    socket_type: Option<SocketType>,

    peer_address: SocketAddr,

    remote_addr: Option<Address>,

    protocol_name: Option<String>,

    is_closed: Arc<AtomicBool>,

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
            is_closed: Arc::new(AtomicBool::new(false)),
            is_allow_proxy: true,
        }
    }

    pub fn set_socket_type(&mut self, socket_type: SocketType) {
        self.socket_type = Some(socket_type);
    }

    pub fn set_protocol_name(&mut self, protocol_name: &str) {
        self.protocol_name = Some(protocol_name.to_string());
    }

    pub fn set_allow_proxy(&mut self, allow: bool) {
        self.is_allow_proxy = allow;
    }

    pub async fn start_connect(&mut self, resolved: &ResolvedResult) -> Result<()> {
        let socket_type = self.socket_type.as_ref().unwrap();
        let protocol_name = self.protocol_name.as_ref().unwrap();
        let peer_address = self.peer_address;

        log::info!(
            "[{}] [{}] use [{}] for outbound",
            peer_address,
            socket_type,
            protocol_name
        );

        let remote_addr = self.get_actual_remote_addr(resolved);

        self.protocol_name = Some(protocol_name.to_string());
        self.remote_addr = Some(remote_addr.clone());

        // resolve dest ip address
        let remote_ip_addr = dns_resolve(&remote_addr).await.map_err(|err| {
            let msg = format!(
                "[{}] [{}] resolve ip address of {} failed due to: {}",
                peer_address, socket_type, remote_addr, err
            );
            log::error!("{}", msg);
            Error::msg(msg)
        })?;

        let target_str = if remote_addr.is_hostname() {
            format!("{}({})", remote_addr, remote_ip_addr)
        } else {
            format!("{}", remote_addr)
        };

        log::info!("[{}] [{}] connecting to {}...", peer_address, socket_type, target_str);

        // make connection
        let socket = self.connect(&remote_addr, remote_ip_addr).await.map_err(|err| {
            let msg = format!(
                "[{}] [{}] connect to {} failed due to: {}",
                peer_address, socket_type, target_str, err
            );
            log::error!("{}", msg);
            Error::msg(msg)
        })?;

        self.socket = Some(socket);

        log::info!("[{}] [{}] connected to {}", peer_address, socket_type, target_str);

        Ok(())
    }

    pub fn handle_incoming_data(&self, mut in_proto: DynProtocol, mut out_proto: DynProtocol, tx: Sender<Event>) {
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
        let is_closed = self.is_closed.clone();

        tokio::spawn(async move {
            loop {
                if is_closed.load(Ordering::Relaxed) {
                    break;
                }

                // protocol process
                let res = match service_type {
                    ServiceType::Client => out_proto.client_decode(&socket).await,
                    ServiceType::Server => in_proto.server_encode(&socket).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::OutboundError(err)).await;
                    break;
                }

                let buf = res.unwrap();

                // send data out
                let event = match service_type {
                    ServiceType::Client => Event::ClientDecodeDone(buf),
                    ServiceType::Server => Event::ServerEncodeDone(buf),
                };

                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
    }

    pub async fn send(&self, buf: Bytes) -> tokio::io::Result<()> {
        let socket = self.socket.as_ref().unwrap();
        socket.send(&buf).await
    }

    pub async fn close(&mut self) -> Result<()> {
        if !self.is_closed.load(Ordering::Relaxed) {
            if let Some(socket) = self.socket.as_ref() {
                socket.close().await?;
            }
        }
        self.is_closed.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn get_actual_remote_addr(&self, resolved: &ResolvedResult) -> Address {
        if self.opts.is_server() || self.opts.server_bind().is_none() || !self.is_allow_proxy {
            resolved.address.clone()
        } else {
            self.opts.server_bind().unwrap()
        }
    }

    async fn connect(&self, addr: &Address, ip_addr: SocketAddr) -> Result<Arc<Socket>> {
        let socket_type = self.socket_type.as_ref().unwrap();
        let peer_address = self.peer_address;

        let socket = match socket_type {
            SocketType::Tcp | SocketType::Tls => {
                #[cfg(target_os = "linux")]
                use std::os::unix::io::AsRawFd;

                let socket = match ip_addr {
                    SocketAddr::V4(..) => TcpSocket::new_v4()?,
                    SocketAddr::V6(..) => TcpSocket::new_v6()?,
                };

                #[cfg(target_os = "linux")]
                if self.opts.is_client() {
                    if let Err(err) = self.mark_socket(socket.as_raw_fd(), 0xff) {
                        log::error!("set SO_MARK error due to: {}", err);
                    }
                }

                let future = socket.connect(ip_addr);
                let tcp_stream = timeout(Duration::from_secs(constants::TCP_CONNECT_TIMEOUT_SECONDS), future).await??;

                match socket_type {
                    SocketType::Tcp => Arc::new(Socket::from_tcp_stream(tcp_stream)),
                    SocketType::Tls => {
                        // create TlsStream from TcpStream
                        let connector = TlsConnector::from(Arc::new(get_tls_client_config()));
                        let domain = rustls::ServerName::try_from(addr.host().as_str())?;

                        let tls_stream = connector.connect(domain, tcp_stream).await?;

                        Arc::new(Socket::from_tls_stream(TlsStream::Client(tls_stream)))
                    }
                    _ => unreachable!(),
                }
            }
            SocketType::Udp => {
                let socket = Socket::bind_udp_random_port(ip_addr).await?;
                Arc::new(socket)
            }
            SocketType::Quic => {
                let RandomEndpoint { inner: endpoint, reuse } = global::get_random_endpoint()?;

                if reuse {
                    log::info!(
                        "[{}] [{}] reuse endpoint, local_port = {}",
                        peer_address,
                        socket_type,
                        endpoint.local_addr()?.port()
                    );
                }

                let future = endpoint.connect(ip_addr, &addr.host())?;
                let conn = timeout(Duration::from_secs(constants::QUIC_CONNECT_TIMEOUT_SECONDS), future).await??;

                let conn = conn.connection;
                let stream = conn.open_bi().await.unwrap();
                let peer_addr = conn.remote_address();

                log::info!(
                    "[{}] [{}] connection RTT = {}ms",
                    peer_address,
                    socket_type,
                    conn.rtt().as_millis()
                );

                Arc::new(Socket::from_quic(peer_addr, stream))
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
