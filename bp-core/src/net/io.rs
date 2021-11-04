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

use crate::config;

/// SocketReader
#[derive(Debug)]
pub struct SocketReader {
    // TODO: refactor cache/restore design
    cache: Mutex<BytesMut>,

    restore_data: Mutex<Vec<Bytes>>,

    restore_data_used: AtomicBool,

    recv_buf: Mutex<BytesMut>,

    tcp_reader: Option<Mutex<ReadHalf<TcpStream>>>,

    udp_reader: Option<Arc<UdpSocket>>,
}

impl SocketReader {
    pub fn new(tcp_reader: Option<Mutex<ReadHalf<TcpStream>>>, udp_socket: Option<Arc<UdpSocket>>) -> Self {
        let cache = Mutex::new(BytesMut::with_capacity(1024));
        let restore_data = Mutex::new(vec![]);
        let restore_data_used = AtomicBool::new(false);
        let recv_buf = Mutex::new(BytesMut::with_capacity(config::RECV_BUFFER_SIZE));

        Self {
            cache,
            restore_data,
            restore_data_used,
            recv_buf,
            tcp_reader,
            udp_reader: udp_socket,
        }
    }

    pub async fn read_some(&self) -> Result<Bytes> {
        let mut recv_buf = self.recv_buf.lock().await;
        self.read_into(&mut recv_buf).await?;

        // TODO: reduce clone to improve performance
        let buf = recv_buf.clone().freeze();
        recv_buf.clear();

        Ok(buf)
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
                return Err(Error::msg("read_buf return 0"));
            }

            if !self.is_restore_data_used() {
                Some(buf.clone().freeze().slice(prev_len..prev_len + n))
            } else {
                None
            }
        } else {
            let new_buf = self.udp_recv().await?;

            if self.is_restore_data_used() {
                buf.put(new_buf);
                None
            } else {
                buf.put(new_buf.clone());
                Some(new_buf)
            }
        };

        if let Some(buf) = data {
            self.store(buf).await;
        }

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
                    let msg = format!(
                        "read_exact error due to: new udp packet size = {} is less than required len = {}",
                        req_buf_len,
                        new_buf.len()
                    );
                    return Err(Error::msg(msg));
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

        let buf: Bytes = final_buf.into();

        if !self.is_restore_data_used() {
            self.store(buf.clone()).await;
        }

        Ok(buf)
    }

    async fn store(&self, buf: Bytes) {
        if self.is_restore_data_used() {
            return;
        }
        let mut restore_data = self.restore_data.lock().await;
        restore_data.push(buf);
    }

    pub async fn restore(&self) {
        let mut restore_data = self.restore_data.lock().await;

        for item in restore_data.iter() {
            self.cache(item.clone()).await;
        }

        restore_data.clear();
    }

    pub async fn clear_restore(&self) {
        self.restore_data.lock().await.clear();
        self.restore_data_used.store(true, Ordering::Relaxed);
    }

    pub async fn cache(&self, buf: Bytes) {
        if buf.is_empty() {
            return;
        }

        let mut cache = self.cache.lock().await;
        let prev = cache.clone().freeze();

        cache.clear();
        cache.put(buf);
        cache.put(prev);
    }

    async fn udp_recv(&self) -> Result<Bytes> {
        let socket = self.udp_reader.as_ref().unwrap();

        let mut buf = vec![0u8; config::UDP_MTU];
        let (len, _addr) = socket.recv_from(&mut buf).await?;

        if let Some(packet) = buf.get(0..len) {
            Ok(Bytes::copy_from_slice(packet))
        } else {
            Err(Error::msg("error recv from remote"))
        }
    }

    fn is_restore_data_used(&self) -> bool {
        self.restore_data_used.load(Ordering::Relaxed)
    }

    fn is_tcp(&self) -> bool {
        self.tcp_reader.is_some()
    }
}

/// TcpStreamWriter
#[derive(Debug)]
pub struct SocketWriter {
    peer_addr: SocketAddr,

    tcp_writer: Option<Mutex<WriteHalf<TcpStream>>>,

    udp_writer: Option<Arc<UdpSocket>>,
}

impl SocketWriter {
    pub fn new(
        tcp_write_half: Option<Mutex<WriteHalf<TcpStream>>>,
        udp_socket: Option<Arc<UdpSocket>>,
        peer_addr: SocketAddr,
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

pub fn split_tcp(stream: TcpStream, peer_addr: SocketAddr) -> (SocketReader, SocketWriter) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = SocketReader::new(Some(Mutex::new(read_half)), None);
    let writer = SocketWriter::new(Some(Mutex::new(write_half)), None, peer_addr);

    (reader, writer)
}

pub fn split_udp(socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> (SocketReader, SocketWriter) {
    let reader = SocketReader::new(None, Some(socket.clone()));
    let writer = SocketWriter::new(None, Some(socket), peer_addr);

    (reader, writer)
}
