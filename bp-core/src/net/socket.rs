use crate::net::io;
use crate::utils::net::create_udp_client_with_random_port;
use crate::Result;
use bytes::Bytes;
use std::fmt::Display;
#[cfg(not(target_os = "windows"))]
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::Arc;
use tokio::net;

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    Tcp,
    Udp,
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SocketType::Tcp => "tcp",
            SocketType::Udp => "udp",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug)]
pub struct Socket {
    #[cfg(not(target_os = "windows"))]
    fd: Option<RawFd>,

    socket_type: SocketType,

    reader: io::SocketReader,

    writer: io::SocketWriter,

    peer_addr: std::net::SocketAddr,

    local_addr: std::net::SocketAddr,
}

impl Socket {
    pub fn new_tcp(stream: net::TcpStream) -> Self {
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();
        #[cfg(not(target_os = "windows"))]
        let fd = stream.as_raw_fd();

        let split = io::split_tcp(stream, peer_addr);

        Self {
            #[cfg(not(target_os = "windows"))]
            fd: Some(fd),
            socket_type: SocketType::Tcp,
            reader: split.0,
            writer: split.1,
            peer_addr,
            local_addr,
        }
    }

    pub fn new_udp(socket: Arc<net::UdpSocket>, peer_addr: std::net::SocketAddr) -> Self {
        let local_addr = socket.local_addr().unwrap();
        let split = io::split_udp(socket, peer_addr);

        Self {
            #[cfg(not(target_os = "windows"))]
            fd: None,
            socket_type: SocketType::Udp,
            reader: split.0,
            writer: split.1,
            peer_addr,
            local_addr,
        }
    }

    pub async fn bind_udp_random_port(peer_addr: std::net::SocketAddr) -> Result<Self> {
        let socket = create_udp_client_with_random_port().await?;
        Ok(Self::new_udp(Arc::new(socket), peer_addr))
    }

    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.peer_addr)
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.local_addr)
    }

    pub fn is_tcp(&self) -> bool {
        match self.socket_type {
            SocketType::Tcp => true,
            SocketType::Udp => false,
        }
    }

    pub fn is_udp(&self) -> bool {
        match self.socket_type {
            SocketType::Tcp => false,
            SocketType::Udp => true,
        }
    }

    pub fn socket_type(&self) -> SocketType {
        self.socket_type
    }
}

impl Socket {
    pub async fn cache(&self, buf: bytes::Bytes) {
        self.reader.cache(buf).await;
    }

    pub async fn read_some(&self) -> Result<Bytes> {
        self.reader.read_some().await
    }

    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        self.reader.read_exact(len).await
    }

    pub async fn read_into(&self, buf: &mut bytes::BytesMut) -> Result<()> {
        self.reader.read_into(buf).await
    }

    pub async fn restore(&self) {
        self.reader.restore().await;
    }

    pub async fn clear_restore(&self) {
        self.reader.clear_restore().await;
    }

    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        self.writer.send(buf).await
    }

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
