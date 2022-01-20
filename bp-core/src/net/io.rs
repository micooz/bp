use std::{
    fmt::Display,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use parking_lot;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpStream, UdpSocket},
    sync::Mutex,
};

use crate::{config, utils::store::Store};

#[derive(Debug, Clone, Copy)]
pub enum SocketType {
    Tcp,
    Udp,
    Quic,
    Unknown,
}

impl Display for SocketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SocketType::Tcp => "tcp",
            SocketType::Udp => "udp",
            SocketType::Quic => "quic",
            SocketType::Unknown => "",
        };
        write!(f, "{}", s)
    }
}

impl Default for SocketType {
    fn default() -> Self {
        Self::Udp
    }
}

#[derive(Debug)]
enum Reader {
    Unknown,
    Tcp(ReadHalf<TcpStream>),
    Udp(Arc<UdpSocket>),
    Quic(quinn::RecvStream),
}

impl Default for Reader {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug)]
enum Writer {
    Unknown,
    Tcp(WriteHalf<TcpStream>),
    Udp(Arc<UdpSocket>),
    Quic(quinn::SendStream),
}

impl Default for Writer {
    fn default() -> Self {
        Self::Unknown
    }
}

/// SocketReader
#[derive(Debug, Default)]
pub struct SocketReader {
    socket_type: SocketType,

    reader: Mutex<Reader>,

    cache: parking_lot::Mutex<Store>,

    restore: parking_lot::Mutex<Store>,

    restore_disabled: AtomicBool,
}

impl SocketReader {
    pub fn from_tcp(tcp_read_half: ReadHalf<TcpStream>) -> Self {
        Self {
            reader: Mutex::new(Reader::Tcp(tcp_read_half)),
            ..Self::default()
        }
    }

    pub fn from_udp(socket: Arc<UdpSocket>) -> Self {
        Self {
            reader: Mutex::new(Reader::Udp(socket)),
            ..Self::default()
        }
    }

    pub fn from_quic(recv_stream: quinn::RecvStream) -> Self {
        Self {
            reader: Mutex::new(Reader::Quic(recv_stream)),
            ..Self::default()
        }
    }

    pub async fn read_some(&self) -> Result<Bytes> {
        let mut recv_buf = BytesMut::with_capacity(config::RECV_BUFFER_SIZE);
        let n = self.read_into(&mut recv_buf).await?;
        Ok(recv_buf.copy_to_bytes(n))
    }

    pub async fn read_into(&self, out_buf: &mut BytesMut) -> Result<usize> {
        if !self.is_cache_empty() {
            let mut cache = self.cache.lock();

            let data = cache.pull_all();
            let data_len = data.len();

            self.store(|| data.clone());

            out_buf.put(data);

            return Ok(data_len);
        }

        match self.socket_type {
            SocketType::Tcp | SocketType::Quic => {
                let prev_len = out_buf.len();

                let n = match &mut *self.reader.lock().await {
                    Reader::Tcp(reader) => reader.read_buf(out_buf).await?,
                    Reader::Quic(reader) => reader.read_buf(out_buf).await?,
                    _ => unreachable!(),
                };

                if n == 0 {
                    return Err(Error::msg("read_buf return 0"));
                }

                self.store(|| {
                    let buf = out_buf.clone().freeze();
                    let buf = buf.slice(prev_len..prev_len + n);
                    buf
                });

                Ok(n)
            }
            SocketType::Udp => {
                let (buf, n) = self.udp_recv().await?;

                self.store(|| buf.clone());

                out_buf.put(buf);

                Ok(n)
            }
            SocketType::Unknown => unreachable!(),
        }
    }

    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        let cache_len = self.cache_len();

        // cached data is not enough
        if len > cache_len {
            match self.socket_type {
                SocketType::Tcp | SocketType::Quic => {
                    let mut req_buf = vec![0u8; len - cache_len];

                    match &mut *self.reader.lock().await {
                        Reader::Tcp(reader) => {
                            reader.read_exact(&mut req_buf).await?;
                        }
                        Reader::Quic(reader) => {
                            reader.read_exact(&mut req_buf).await?;
                        }
                        _ => unreachable!(),
                    };

                    let mut cache = self.cache.lock();
                    cache.push_back(req_buf.into());
                }
                SocketType::Udp => {
                    let req_buf_len = len - cache_len;
                    let (packet, size) = self.udp_recv().await?;

                    if size < req_buf_len {
                        let msg = format!(
                            "read_exact error due to: new udp packet size = {} is less than required len = {}",
                            size, req_buf_len,
                        );
                        return Err(Error::msg(msg));
                    }

                    let mut cache = self.cache.lock();
                    cache.push_back(packet);
                }
                SocketType::Unknown => unreachable!(),
            }
        }

        let mut cache = self.cache.lock();
        let buf = cache.pull(len);

        self.store(|| buf.clone());

        Ok(buf)
    }

    pub fn restore(&self) {
        let mut cache = self.cache.lock();
        let mut restore = self.restore.lock();

        while let Some(item) = restore.pop_back() {
            cache.push_front(item);
        }
    }

    pub fn disable_restore(&self) {
        self.restore.lock().clear();
        self.restore_disabled.store(true, Ordering::Relaxed);
    }

    pub fn cache(&self, buf: Bytes) {
        if buf.is_empty() {
            return;
        }
        let mut cache = self.cache.lock();
        cache.push_back(buf);
    }

    async fn udp_recv(&self) -> Result<(Bytes, usize)> {
        if let Reader::Udp(socket) = &*self.reader.lock().await {
            let mut buf = vec![0u8; config::UDP_MTU];
            let (len, _addr) = socket.recv_from(&mut buf).await?;

            if let Some(packet) = buf.get(0..len) {
                return Ok((Bytes::copy_from_slice(packet), packet.len()));
            } else {
                return Err(Error::msg("error recv from remote"));
            }
        }
        unreachable!()
    }

    #[inline]
    fn cache_len(&self) -> usize {
        let cache = self.cache.lock();
        cache.len()
    }

    #[inline]
    fn is_cache_empty(&self) -> bool {
        let cache = self.cache.lock();
        cache.is_empty()
    }

    #[inline]
    fn store<F: Fn() -> Bytes>(&self, closure: F) {
        if !self.is_restore_disabled() {
            let mut restore = self.restore.lock();
            restore.push_back(closure());
        }
    }

    #[inline]
    fn is_restore_disabled(&self) -> bool {
        self.restore_disabled.load(Ordering::Relaxed)
    }
}

/// SocketWriter
#[derive(Debug, Default)]
pub struct SocketWriter {
    socket_type: SocketType,

    writer: Mutex<Writer>,

    peer_addr: Option<SocketAddr>,
}

impl SocketWriter {
    pub fn from_tcp(write_half: WriteHalf<TcpStream>) -> Self {
        Self {
            writer: Mutex::new(Writer::Tcp(write_half)),
            ..Self::default()
        }
    }

    pub fn from_udp(socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> Self {
        Self {
            peer_addr: Some(peer_addr),
            writer: Mutex::new(Writer::Udp(socket)),
            ..Self::default()
        }
    }

    pub fn from_quic(send_stream: quinn::SendStream) -> Self {
        Self {
            writer: Mutex::new(Writer::Quic(send_stream)),
            ..Self::default()
        }
    }

    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        match self.socket_type {
            SocketType::Tcp | SocketType::Quic => {
                match &mut *self.writer.lock().await {
                    Writer::Tcp(writer) => {
                        writer.write_all(buf).await?;
                        writer.flush().await?;
                    }
                    Writer::Quic(writer) => {
                        writer.write_all(buf).await?;
                        writer.flush().await?;
                    }
                    _ => unreachable!(),
                };
            }
            SocketType::Udp => {
                if let Writer::Udp(writer) = &*self.writer.lock().await {
                    writer.send_to(buf, self.peer_addr.as_ref().unwrap()).await?;
                }
            }
            SocketType::Unknown => unreachable!(),
        }

        Ok(())
    }

    pub async fn close(&self) -> tokio::io::Result<()> {
        match self.socket_type {
            SocketType::Tcp | SocketType::Quic => {
                match &mut *self.writer.lock().await {
                    Writer::Tcp(writer) => {
                        writer.shutdown().await?;
                    }
                    Writer::Quic(writer) => {
                        writer.shutdown().await?;
                    }
                    _ => unreachable!(),
                };
            }
            SocketType::Udp => {}
            SocketType::Unknown => unreachable!(),
        }

        Ok(())
    }
}

pub fn split_tcp(stream: TcpStream) -> (SocketReader, SocketWriter) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = SocketReader::from_tcp(read_half);
    let writer = SocketWriter::from_tcp(write_half);

    (reader, writer)
}

pub fn split_udp(socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> (SocketReader, SocketWriter) {
    let reader = SocketReader::from_udp(socket.clone());
    let writer = SocketWriter::from_udp(socket, peer_addr);

    (reader, writer)
}

pub async fn split_quic(conn: quinn::Connection) -> (SocketReader, SocketWriter) {
    let (send, recv) = conn.open_bi().await.unwrap();

    let reader = SocketReader::from_quic(recv);
    let writer = SocketWriter::from_quic(send);

    (reader, writer)
}
