use crate::{config, event, net, net::socket, protocol, Options, Result};
use bytes::Bytes;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpSocket, time};

type Proto = protocol::DynProtocol;

pub struct Outbound {
    opts: Options,

    socket: Option<Arc<socket::Socket>>,

    socket_type: socket::SocketType,

    peer_address: SocketAddr,

    remote_addr: Option<net::Address>,

    protocol_name: Option<String>,

    is_proxy: bool,
}

impl Outbound {
    pub fn new(peer_address: SocketAddr, socket_type: socket::SocketType, opts: Options) -> Self {
        Self {
            opts,
            socket: None,
            socket_type,
            peer_address,
            remote_addr: None,
            protocol_name: None,
            is_proxy: false,
        }
    }

    // apply transport protocol then make connection to remote
    pub async fn use_protocol(
        &mut self,
        mut out_proto: Proto,
        mut in_proto: Proto,
        tx: event::EventSender,
    ) -> Result<()> {
        let service_type = self.opts.service_type();
        let socket_type = self.socket_type;
        let peer_address = self.peer_address;

        let protocol_name = out_proto.get_name();
        self.protocol_name = Some(protocol_name.clone());

        log::info!(
            "[{}] [{}] use [{}] protocol",
            self.peer_address,
            socket_type,
            protocol_name
        );

        let remote_addr = self.get_remote_addr(&in_proto);
        self.remote_addr = Some(remote_addr.clone());

        log::info!("[{}] [{}] connecting to {}...", peer_address, socket_type, remote_addr,);

        // resolve target ip address
        let remote_ip_addr = if !remote_addr.is_ip() {
            // dns resolve
            let ip_list = remote_addr.dns_resolve().await;

            log::info!(
                "[{}] [{}] resolved {} to {:?}",
                peer_address,
                socket_type,
                remote_addr,
                ip_list
            );

            ip_list[0]
        } else {
            remote_addr.as_socket_addr()
        };

        // make connection
        let socket = self.connect(remote_ip_addr).await.map_err(|err| {
            format!(
                "[{}] [{}] connect to {} failed due to {}",
                peer_address, socket_type, remote_addr, err
            )
        })?;

        self.socket = Some(socket.clone());

        log::info!("[{}] [{}] connected to {}", peer_address, socket_type, remote_addr);

        log::info!(
            "[{}] [{}] [{}] start relaying data...",
            peer_address,
            socket_type,
            protocol_name
        );

        // start receiving data from outbound
        tokio::spawn(async move {
            loop {
                let future = match service_type {
                    net::ServiceType::Client => out_proto.client_decode(&socket, tx.clone()),
                    net::ServiceType::Server => in_proto.server_encode(&socket, tx.clone()),
                };

                let res = time::timeout(time::Duration::from_secs(config::OUTBOUND_RECV_TIMEOUT_SECONDS), future).await;

                match res {
                    Ok(res) => {
                        if let Err(err) = res {
                            let _ = tx.send(event::Event::OutboundError(err)).await;
                            break;
                        }
                    }
                    Err(err) => {
                        log::warn!(
                            "[{}] [{}] [{}] no data received from outbound for {} seconds",
                            peer_address,
                            socket_type,
                            protocol_name,
                            config::OUTBOUND_RECV_TIMEOUT_SECONDS
                        );
                        let _ = tx.send(event::Event::OutboundError(Box::new(err))).await;
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// send data to remote
    pub async fn send(&self, buf: Bytes) -> tokio::io::Result<()> {
        let socket = self.socket.as_ref().unwrap();

        if socket.is_udp() {
            log::info!(
                "[{}] [{}] [{}] sent an udp packet: {} bytes",
                self.peer_address,
                self.socket_type,
                self.protocol_name.as_ref().unwrap(),
                buf.len()
            );
        }

        socket.send(&buf).await
    }

    /// close the bound
    pub async fn close(&mut self) -> Result<()> {
        if let Some(socket) = self.socket.as_ref() {
            socket.close().await?;
        }

        Ok(())
    }

    pub fn snapshot(&self) -> OutboundSnapshot {
        OutboundSnapshot {
            remote_addr: self.remote_addr.clone(),
            protocol_name: self.protocol_name.clone(),
        }
    }

    pub fn set_is_proxy(&mut self, is_proxy: bool) {
        self.is_proxy = is_proxy;
    }

    fn get_remote_addr(&self, in_proto: &Proto) -> net::Address {
        let service_type = self.opts.service_type();

        if service_type.is_server() || self.opts.server_bind.is_none() || !self.is_proxy {
            in_proto.get_proxy_address().unwrap()
        } else {
            self.opts.server_bind.as_ref().unwrap().clone()
        }
    }

    async fn connect(&self, addr: SocketAddr) -> Result<Arc<socket::Socket>> {
        let socket = match self.socket_type {
            socket::SocketType::Tcp => {
                #[cfg(target_os = "linux")]
                use std::os::unix::io::AsRawFd;

                let socket = match addr {
                    SocketAddr::V4(..) => TcpSocket::new_v4()?,
                    SocketAddr::V6(..) => TcpSocket::new_v6()?,
                };

                #[cfg(target_os = "linux")]
                if let Err(err) = self.mark_socket(socket.as_raw_fd(), 0xff) {
                    log::warn!("set SO_MARK error due to: {}", err);
                }

                let future = socket.connect(addr);
                let stream =
                    time::timeout(time::Duration::from_secs(config::TCP_CONNECT_TIMEOUT_SECONDS), future).await??;

                Arc::new(socket::Socket::new_tcp(stream))
            }
            socket::SocketType::Udp => {
                let socket = socket::Socket::bind_udp_random_port(addr).await?;
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
            return Err(Box::new(err));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct OutboundSnapshot {
    pub remote_addr: Option<net::Address>,
    pub protocol_name: Option<String>,
}
