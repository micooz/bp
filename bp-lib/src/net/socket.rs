use crate::net::io;
use crate::Result;
use bytes::BufMut;
use std::fmt::Display;
use std::sync::Arc;
use std::os::unix::io::{AsRawFd, RawFd};
use tokio::net;
use tokio::sync::Mutex;

#[derive(Clone, Copy)]
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

pub struct Socket {
    tcp_socket_fd: Option<RawFd>,

    tcp_reader: Option<Arc<Mutex<io::TcpStreamReader>>>,

    tcp_writer: Option<Arc<Mutex<io::TcpStreamWriter>>>,

    udp_socket_wrapper: Option<UdpSocketWrapper>,

    peer_addr: std::net::SocketAddr,

    local_addr: std::net::SocketAddr,

    udp_cache: Option<Mutex<bytes::BytesMut>>,
}

impl Socket {
    pub fn new_tcp(stream: net::TcpStream) -> Self {
        let peer_addr = stream.peer_addr().unwrap();
        let local_addr = stream.local_addr().unwrap();
        let tcp_socket_fd  = stream.as_raw_fd();

        let split = io::split_tcp_stream(stream);

        Self {
            tcp_socket_fd: Some(tcp_socket_fd),
            tcp_reader: Some(split.0),
            tcp_writer: Some(split.1),
            udp_socket_wrapper: None,
            udp_cache: None,
            peer_addr,
            local_addr,
        }
    }

    pub fn new_udp(socket: UdpSocketWrapper) -> Self {
        let peer_addr = socket.peer_address;
        let local_addr = socket.inner.local_addr().unwrap();

        Self {
            tcp_socket_fd: None,
            tcp_reader: None,
            tcp_writer: None,
            udp_socket_wrapper: Some(socket),
            udp_cache: Some(Mutex::new(bytes::BytesMut::with_capacity(32))),
            peer_addr,
            local_addr,
        }
    }

    pub fn get_tcp_socket_fd(&self) -> RawFd {
        self.tcp_socket_fd.unwrap()
    }

    pub fn peer_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.peer_addr)
    }

    pub fn local_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        Ok(self.local_addr)
    }

    pub fn tcp_reader(&self) -> Arc<Mutex<io::TcpStreamReader>> {
        self.tcp_reader.clone().unwrap()
    }

    pub fn tcp_writer(&self) -> Arc<Mutex<io::TcpStreamWriter>> {
        self.tcp_writer.clone().unwrap()
    }

    pub fn is_tcp(&self) -> bool {
        self.tcp_reader.is_some() && self.tcp_writer.is_some()
    }

    pub fn is_udp(&self) -> bool {
        self.udp_socket_wrapper.is_some()
    }

    pub fn socket_type(&self) -> SocketType {
        if self.is_tcp() {
            SocketType::Tcp
        } else {
            SocketType::Udp
        }
    }

    pub fn udp_packet(&self) -> Option<bytes::Bytes> {
        let socket = self.udp_socket_wrapper.as_ref().unwrap();
        socket.packet.clone()
    }

    pub async fn cache(&self, buf: bytes::Bytes) {
        if self.is_tcp() {
            let reader = self.tcp_reader();
            let mut reader = reader.lock().await;
            reader.cache(buf);
        } else {
            let mut cache = self.udp_cache.as_ref().unwrap().lock().await;
            cache.put(buf);
        }
    }

    pub async fn read_buf(&self, capacity: usize) -> Result<bytes::Bytes> {
        if self.is_tcp() {
            let reader = self.tcp_reader();
            let mut reader = reader.lock().await;

            reader.read_buf(capacity).await
        } else {
            let mut cache = self.udp_cache.as_ref().unwrap().lock().await;

            if !cache.is_empty() {
                let buf = cache.clone().freeze();
                cache.clear();
                return Ok(buf);
            }

            let socket = self.udp_socket_wrapper.as_ref().unwrap();
            let buf = socket.recv().await?;

            Ok(buf)
        }
    }

    pub async fn read_exact(&self, len: usize) -> Result<bytes::Bytes> {
        let reader = self.tcp_reader();
        let mut reader = reader.lock().await;
        reader.read_exact(len).await
    }

    pub async fn send(&self, buf: &[u8]) -> Result<()> {
        if self.is_tcp() {
            let writer = self.tcp_writer();
            let mut writer = writer.lock().await;

            writer.write_all(buf).await?;
            writer.flush().await?;
        } else {
            let socket = self.udp_socket_wrapper.as_ref().unwrap();
            socket.send(buf).await?;
        }

        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        if self.is_tcp() {
            let writer = self.tcp_writer();
            let mut writer = writer.lock().await;

            // only close the write half
            writer.shutdown().await?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct UdpSocketWrapper {
    inner: Arc<net::UdpSocket>,
    peer_address: std::net::SocketAddr,
    packet: Option<bytes::Bytes>,
}

impl UdpSocketWrapper {
    pub fn new(socket: Arc<net::UdpSocket>, peer_addr: std::net::SocketAddr) -> Self {
        Self {
            inner: socket,
            peer_address: peer_addr,
            packet: None,
        }
    }

    pub async fn bind_random_port(peer_addr: std::net::SocketAddr) -> Result<Self> {
        use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

        let mut max_retry_times = 10u8;
        let mut rng = StdRng::from_rng(thread_rng()).unwrap();

        loop {
            let port: u32 = rng.gen_range(10000..65535);
            let bind_addr = format!("0.0.0.0:{}", port);

            match net::UdpSocket::bind(bind_addr).await {
                Ok(socket) => {
                    return Ok(Self {
                        inner: Arc::new(socket),
                        peer_address: peer_addr,
                        packet: None,
                    });
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

    pub fn set_packet(&mut self, data: &[u8]) {
        self.packet = Some(bytes::Bytes::copy_from_slice(data));
    }

    pub async fn send(&self, buf: &[u8]) -> Result<()> {
        self.inner.send_to(buf, self.peer_address).await?;
        Ok(())
    }

    pub async fn recv(&self) -> Result<bytes::Bytes> {
        let mut buf = vec![0u8; 1500];
        let (len, _addr) = self.inner.recv_from(&mut buf).await?;

        if let Some(packet) = buf.get(0..len) {
            Ok(bytes::Bytes::copy_from_slice(packet))
        } else {
            Err("error recv from remote".into())
        }
    }
}