use crate::config;
use crate::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net, sync,
};

/// SocketReader
#[derive(Debug)]
pub struct SocketReader {
    cache: sync::Mutex<BytesMut>,

    restore_data: sync::Mutex<bytes::BytesMut>,

    tcp_reader: Option<sync::Mutex<ReadHalf<net::TcpStream>>>,

    udp_reader: Option<Arc<net::UdpSocket>>,
}

impl SocketReader {
    pub fn new(
        tcp_read_half: Option<sync::Mutex<ReadHalf<net::TcpStream>>>,
        udp_socket: Option<Arc<net::UdpSocket>>,
    ) -> Self {
        Self {
            cache: sync::Mutex::new(BytesMut::with_capacity(1024)),
            restore_data: sync::Mutex::new(bytes::BytesMut::with_capacity(64)),
            tcp_reader: tcp_read_half,
            udp_reader: udp_socket,
        }
    }

    pub async fn read_buf(&self, capacity: usize) -> Result<Bytes> {
        let mut buf = BytesMut::with_capacity(capacity);
        self.read_into(&mut buf).await?;

        Ok(buf.freeze())
    }

    pub async fn read_into(&self, buf: &mut BytesMut) -> Result<()> {
        let mut cache = self.cache.lock().await;

        if !cache.is_empty() {
            let data = cache.clone().freeze();
            buf.put(data.clone());
            cache.clear();

            self.store(data).await;
            return Ok(());
        }

        let data = if self.is_tcp() {
            let mut reader = self.tcp_reader.as_ref().unwrap().lock().await;

            let prev_len = buf.len();
            let n = reader.read_buf(buf).await?;

            if n == 0 {
                return Err("read_buf return 0".into());
            }

            buf.clone().freeze().slice(prev_len..prev_len + n)
        } else {
            let new_buf = self.udp_recv().await?;
            buf.put(new_buf.clone());
            new_buf
        };

        self.store(data).await;

        Ok(())
    }

    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        let mut cache = self.cache.lock().await;
        let cache_len = cache.len();

        let final_buf = if len > cache_len {
            // cached buffer is not enough

            if self.is_tcp() {
                let mut req_buf = vec![0u8; len - cache_len];
                let mut reader = self.tcp_reader.as_ref().unwrap().lock().await;

                reader.read_exact(&mut req_buf).await?;

                let mut ret_buf = req_buf;

                // with cache
                if cache_len > 0 {
                    ret_buf = [&cache[..], ret_buf.as_slice()].concat();
                    cache.clear();
                }

                ret_buf
            } else {
                let req_buf_len = len - cache_len;
                let new_buf = self.udp_recv().await?;

                if new_buf.len() < req_buf_len {
                    return Err(format!(
                        "read_exact error due to: new udp packet size = {} is less than required len = {}",
                        req_buf_len,
                        new_buf.len()
                    )
                    .into());
                }

                let req_buf = new_buf.slice(0..req_buf_len);

                // cache rest buffer
                let rest_buf = new_buf.slice(req_buf_len..);

                let ret = [&cache[..], &req_buf[..]].concat();

                cache.clear();
                cache.put(rest_buf);

                ret
            }
        } else {
            let (left, _) = cache.split_at(len);
            let buf = left.to_vec();
            cache.advance(len);

            buf
        };

        let buf: bytes::Bytes = final_buf.into();

        self.store(buf.clone()).await;

        Ok(buf)
    }

    async fn store(&self, buf: bytes::Bytes) {
        let mut restore_data = self.restore_data.lock().await;
        restore_data.put(buf);
    }

    pub async fn restore(&self) {
        let mut restore_data = self.restore_data.lock().await;
        self.cache(restore_data.clone().freeze()).await;
        restore_data.clear();
    }

    pub async fn clear_restore(&self) {
        self.restore_data.lock().await.clear();
    }

    pub async fn cache(&self, buf: bytes::Bytes) {
        if buf.is_empty() {
            return;
        }

        let mut cache = self.cache.lock().await;
        let prev = cache.clone().freeze();

        cache.clear();
        cache.put(buf);
        cache.put(prev);
    }

    async fn udp_recv(&self) -> Result<bytes::Bytes> {
        let socket = self.udp_reader.as_ref().unwrap();

        let mut buf = vec![0u8; config::UDP_MTU];
        let (len, _addr) = socket.recv_from(&mut buf).await?;

        if let Some(packet) = buf.get(0..len) {
            Ok(bytes::Bytes::copy_from_slice(packet))
        } else {
            Err("error recv from remote".into())
        }
    }

    fn is_tcp(&self) -> bool {
        self.tcp_reader.is_some()
    }
}

/// TcpStreamWriter
#[derive(Debug)]
pub struct SocketWriter {
    peer_addr: std::net::SocketAddr,

    tcp_writer: Option<sync::Mutex<WriteHalf<net::TcpStream>>>,

    udp_writer: Option<Arc<net::UdpSocket>>,
}

impl SocketWriter {
    pub fn new(
        tcp_write_half: Option<sync::Mutex<WriteHalf<net::TcpStream>>>,
        udp_socket: Option<Arc<net::UdpSocket>>,
        peer_addr: std::net::SocketAddr,
    ) -> Self {
        Self {
            peer_addr,
            tcp_writer: tcp_write_half,
            udp_writer: udp_socket,
        }
    }

    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        if self.is_tcp() {
            let mut writer = self.tcp_writer.as_ref().unwrap().lock().await;
            writer.write_all(buf).await?;
            writer.flush().await?;
        } else {
            let writer = self.udp_writer.as_ref().unwrap();
            writer.send_to(buf, self.peer_addr).await?;
        }
        Ok(())
    }

    pub async fn close(&self) -> tokio::io::Result<()> {
        if self.is_tcp() {
            let mut writer = self.tcp_writer.as_ref().unwrap().lock().await;
            writer.shutdown().await?;
        }
        Ok(())
    }

    fn is_tcp(&self) -> bool {
        self.tcp_writer.is_some()
    }
}

pub fn split_tcp(stream: net::TcpStream, peer_addr: std::net::SocketAddr) -> (SocketReader, SocketWriter) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = SocketReader::new(Some(sync::Mutex::new(read_half)), None);
    let writer = SocketWriter::new(Some(sync::Mutex::new(write_half)), None, peer_addr);

    (reader, writer)
}

pub fn split_udp(socket: Arc<net::UdpSocket>, peer_addr: std::net::SocketAddr) -> (SocketReader, SocketWriter) {
    let reader = SocketReader::new(None, Some(socket.clone()));
    let writer = SocketWriter::new(None, Some(socket), peer_addr);

    (reader, writer)
}
