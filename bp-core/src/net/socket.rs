#[cfg(not(target_os = "windows"))]
use std::os::unix::io::{AsRawFd, RawFd};
use std::{fmt::Display, net::SocketAddr, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use tokio::net;

use crate::{
    io::{
        reader::SocketReader,
        utils::{split_quic, split_tcp, split_udp},
        writer::SocketWriter,
    },
    utils::net::create_udp_client_with_random_port,
};

#[derive(Debug)]
pub struct Socket {
    #[cfg(not(target_os = "windows"))]
    fd: Option<RawFd>,

    socket_type: SocketType,

    reader: SocketReader,

    writer: SocketWriter,

    local_addr: Option<SocketAddr>,

    peer_addr: SocketAddr,
}

impl Socket {
    pub fn from_stream(stream: net::TcpStream) -> Self {
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();

        #[cfg(not(target_os = "windows"))]
        let fd = stream.as_raw_fd();

        let split = split_tcp(stream);

        Self {
            #[cfg(not(target_os = "windows"))]
            fd: Some(fd),
            socket_type: SocketType::Tcp,
            reader: split.0,
            writer: split.1,
            local_addr: Some(local_addr),
            peer_addr,
        }
    }

    pub fn from_quic(peer_addr: SocketAddr, stream: (quinn::SendStream, quinn::RecvStream)) -> Self {
        let (reader, writer) = split_quic(stream);

        Self {
            fd: None,
            socket_type: SocketType::Quic,
            reader,
            writer,
            local_addr: None,
            peer_addr,
        }
    }

    pub fn from_udp_socket(socket: Arc<net::UdpSocket>, peer_addr: SocketAddr) -> Self {
        let local_addr = socket.local_addr().unwrap();
        let split = split_udp(socket);

        Self {
            #[cfg(not(target_os = "windows"))]
            fd: None,
            socket_type: SocketType::Udp,
            reader: split.0,
            writer: split.1,
            local_addr: Some(local_addr),
            peer_addr,
        }
    }

    pub async fn bind_udp_random_port(peer_addr: SocketAddr) -> Result<Self> {
        let socket = create_udp_client_with_random_port().await?;
        Ok(Self::from_udp_socket(Arc::new(socket), peer_addr))
    }

    #[inline]
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    #[inline]
    pub fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    #[inline]
    pub fn is_udp(&self) -> bool {
        matches!(self.socket_type, SocketType::Udp)
    }

    #[inline]
    pub fn socket_type(&self) -> SocketType {
        self.socket_type
    }
}

impl Socket {
    #[inline]
    pub fn cache(&self, buf: bytes::Bytes) {
        self.reader.cache(buf);
    }

    #[inline]
    pub fn restore(&self) {
        self.reader.restore();
    }

    #[inline]
    pub fn disable_restore(&self) {
        self.reader.disable_restore();
    }

    #[inline]
    pub async fn read_some(&self) -> Result<Bytes> {
        self.reader.read_some().await
    }

    #[inline]
    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        self.reader.read_exact(len).await
    }

    #[inline]
    pub async fn read_into(&self, buf: &mut bytes::BytesMut) -> Result<usize> {
        self.reader.read_into(buf).await
    }

    #[inline]
    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        if self.is_udp() {
            self.writer.send_to(buf, self.peer_addr()).await
        } else {
            self.writer.send(buf).await
        }
    }

    #[inline]
    pub async fn send_to(&self, buf: &[u8], peer_addr: SocketAddr) -> tokio::io::Result<()> {
        self.writer.send_to(buf, peer_addr).await
    }

    #[inline]
    pub async fn close(&self) -> tokio::io::Result<()> {
        self.writer.close().await
    }
}

#[cfg(not(target_os = "windows"))]
impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    Tcp,
    Udp,
    Quic,
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SocketType::Tcp => "tcp",
            SocketType::Udp => "udp",
            SocketType::Quic => "quic",
        };
        write!(f, "{}", s)
    }
}
