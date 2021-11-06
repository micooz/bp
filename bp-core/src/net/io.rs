use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use anyhow::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpStream, UdpSocket},
    sync::Mutex,
};

use crate::{config, utils::cache::Cache};

/// SocketReader
#[derive(Debug)]
pub struct SocketReader {
    cache: Mutex<Cache>,

    cache_disabled: AtomicBool,

    recv_buf: Mutex<BytesMut>,

    tcp_reader: Option<Mutex<ReadHalf<TcpStream>>>,

    udp_reader: Option<Arc<UdpSocket>>,
}

impl SocketReader {
    pub fn new(tcp_reader: Option<Mutex<ReadHalf<TcpStream>>>, udp_socket: Option<Arc<UdpSocket>>) -> Self {
        let cache = Mutex::new(Cache::default());
        let cache_disabled = AtomicBool::new(false);
        let recv_buf = Mutex::new(BytesMut::with_capacity(config::RECV_BUFFER_SIZE));

        Self {
            cache,
            cache_disabled,
            recv_buf,
            tcp_reader,
            udp_reader: udp_socket,
        }
    }

    pub async fn read_some(&self) -> Result<Bytes> {
        let mut recv_buf = self.recv_buf.lock().await;
        let n = self.read_into(&mut recv_buf).await?;

        let buf = recv_buf.copy_to_bytes(n);
        recv_buf.clear();

        Ok(buf)
    }

    pub async fn read_into(&self, out_buf: &mut BytesMut) -> Result<usize> {
        let mut cache = self.cache.lock().await;

        if !cache.is_empty() {
            let data = cache.pull_all();
            let data_len = data.len();

            out_buf.put(data);

            return Ok(data_len);
        }

        if self.is_tcp() {
            let mut reader = self.tcp_reader.as_ref().unwrap().lock().await;

            let prev_len = out_buf.len();
            let n = reader.read_buf(out_buf).await?;

            if n == 0 {
                return Err(Error::msg("read_buf return 0"));
            }

            if !self.is_cache_disabled() {
                let buf = out_buf.clone().freeze().slice(prev_len..prev_len + n);
                cache.push_back(buf);
            }

            Ok(n)
        } else {
            let (buf, n) = self.udp_recv().await?;

            if !self.is_cache_disabled() {
                cache.push_back(buf.clone());
            }

            out_buf.put(buf);

            Ok(n)
        }
    }

    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        let mut cache = self.cache.lock().await;
        let cache_len = cache.len();

        // cached data is not enough
        if len > cache_len {
            if self.is_tcp() {
                let mut reader = self.tcp_reader.as_ref().unwrap().lock().await;
                let mut req_buf = vec![0u8; len - cache_len];

                reader.read_exact(&mut req_buf).await?;

                cache.push_back(req_buf.into());
            } else {
                let req_buf_len = len - cache_len;
                let (packet, size) = self.udp_recv().await?;

                if size < req_buf_len {
                    let msg = format!(
                        "read_exact error due to: new udp packet size = {} is less than required len = {}",
                        size, req_buf_len,
                    );
                    return Err(Error::msg(msg));
                }

                cache.push_back(packet);
            }
        }

        Ok(cache.pull(len))
    }

    pub async fn restore(&self) {
        let mut cache = self.cache.lock().await;
        cache.restore();
    }

    pub async fn clear_restore(&self) {
        self.cache.lock().await.clear();
        self.cache_disabled.store(true, Ordering::Relaxed);
    }

    pub async fn cache(&self, buf: Bytes) {
        if buf.is_empty() {
            return;
        }
        let mut cache = self.cache.lock().await;
        cache.push_back(buf);
    }

    async fn udp_recv(&self) -> Result<(Bytes, usize)> {
        let socket = self.udp_reader.as_ref().unwrap();

        let mut buf = vec![0u8; config::UDP_MTU];
        let (len, _addr) = socket.recv_from(&mut buf).await?;

        if let Some(packet) = buf.get(0..len) {
            Ok((Bytes::copy_from_slice(packet), packet.len()))
        } else {
            Err(Error::msg("error recv from remote"))
        }
    }

    fn is_cache_disabled(&self) -> bool {
        self.cache_disabled.load(Ordering::Relaxed)
    }

    fn is_tcp(&self) -> bool {
        self.tcp_reader.is_some()
    }
}

/// TcpStreamWriter
#[derive(Debug)]
pub struct SocketWriter {
    peer_addr: Option<SocketAddr>,

    tcp_writer: Option<Mutex<WriteHalf<TcpStream>>>,

    udp_writer: Option<Arc<UdpSocket>>,
}

impl SocketWriter {
    pub fn new(
        tcp_write_half: Option<Mutex<WriteHalf<TcpStream>>>,
        udp_socket: Option<Arc<UdpSocket>>,
        peer_addr: Option<SocketAddr>,
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
            writer.send_to(buf, self.peer_addr.as_ref().unwrap()).await?;
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

pub fn split_tcp(stream: TcpStream) -> (SocketReader, SocketWriter) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = SocketReader::new(Some(Mutex::new(read_half)), None);
    let writer = SocketWriter::new(Some(Mutex::new(write_half)), None, None);

    (reader, writer)
}

pub fn split_udp(socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> (SocketReader, SocketWriter) {
    let reader = SocketReader::new(None, Some(socket.clone()));
    let writer = SocketWriter::new(None, Some(socket), Some(peer_addr));

    (reader, writer)
}
