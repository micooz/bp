use crate::net::io;
use crate::Result;
use std::fmt::Display;
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
        let fd = stream.as_raw_fd();

        let split = io::split_tcp(stream, peer_addr);

        Self {
            fd: Some(fd),
            reader: split.0,
            writer: split.1,
            peer_addr,
            local_addr,
            socket_type: SocketType::Tcp,
        }
    }

    pub fn new_udp(socket: Arc<net::UdpSocket>, peer_addr: std::net::SocketAddr) -> Self {
        let local_addr = socket.local_addr().unwrap();
        let split = io::split_udp(socket, peer_addr);

        Self {
            fd: None,
            reader: split.0,
            writer: split.1,
            peer_addr,
            local_addr,
            socket_type: SocketType::Udp,
        }
    }

    pub async fn bind_udp_random_port(peer_addr: std::net::SocketAddr) -> Result<Self> {
        use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

        let mut max_retry_times = 10u8;
        let mut rng = StdRng::from_rng(thread_rng()).unwrap();

        loop {
            let port: u32 = rng.gen_range(10000..65535);
            let bind_addr = format!("0.0.0.0:{}", port);

            match net::UdpSocket::bind(bind_addr).await {
                Ok(socket) => {
                    return Ok(Self::new_udp(Arc::new(socket), peer_addr));
                }
                Err(_) => {
                    max_retry_times -= 1;

                    if max_retry_times == 0 {
                        return Err("udp socket random bind error, max retry times exceed".into());
                    }
                }
            }
        }
    }

    pub fn get_socket_fd(&self) -> Option<RawFd> {
        self.fd
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

    pub async fn cache_size(&self) -> usize {
        self.reader.cache_size().await
    }

    pub async fn read_buf(&self, capacity: usize) -> Result<bytes::Bytes> {
        self.reader.read_buf(capacity).await
    }

    pub async fn read_exact(&self, len: usize) -> Result<bytes::Bytes> {
        self.reader.read_exact(len).await
    }

    pub async fn read_into(&self, buf: &mut bytes::BytesMut) -> Result<()> {
        self.reader.read_into(buf).await
    }

    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        self.writer.send(buf).await
    }

    pub async fn close(&self) -> tokio::io::Result<()> {
        self.writer.close().await
    }
}
