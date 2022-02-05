use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::{Error, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use parking_lot;
use quinn::RecvStream;
use tokio::{
    io::{AsyncReadExt, ReadHalf},
    net::{TcpStream, UdpSocket},
    sync::Mutex,
};
use tokio_rustls::TlsStream;

use crate::{constants, utils::store::Store};

#[derive(Debug)]
enum ReaderType {
    Unknown,
    Tcp(ReadHalf<TcpStream>),
    Tls(ReadHalf<TlsStream<TcpStream>>),
    Udp(Arc<UdpSocket>),
    Quic(RecvStream),
}

impl Default for ReaderType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// SocketReader
#[derive(Debug, Default)]
pub struct SocketReader {
    reader: Mutex<ReaderType>,
    cache: parking_lot::Mutex<Store>,
    restore: parking_lot::Mutex<Store>,
    restore_disabled: AtomicBool,
}

impl SocketReader {
    pub fn from_tcp(read_half: ReadHalf<TcpStream>) -> Self {
        Self {
            reader: Mutex::new(ReaderType::Tcp(read_half)),
            ..Self::default()
        }
    }

    pub fn from_tls(read_half: ReadHalf<TlsStream<TcpStream>>) -> Self {
        Self {
            reader: Mutex::new(ReaderType::Tls(read_half)),
            ..Self::default()
        }
    }

    pub fn from_udp(socket: Arc<UdpSocket>) -> Self {
        Self {
            reader: Mutex::new(ReaderType::Udp(socket)),
            ..Self::default()
        }
    }

    pub fn from_quic(recv_stream: RecvStream) -> Self {
        Self {
            reader: Mutex::new(ReaderType::Quic(recv_stream)),
            ..Self::default()
        }
    }

    pub async fn read_some(&self) -> Result<Bytes> {
        let mut recv_buf = BytesMut::with_capacity(constants::RECV_BUFFER_SIZE);
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

        macro_rules! read_stream {
            ($reader:ident) => {{
                let prev_len = out_buf.len();
                let n = $reader.read_buf(out_buf).await?;
                if n == 0 {
                    return Err(Error::msg("read_buf return 0"));
                }
                self.store(|| {
                    let buf = out_buf.clone().freeze();
                    let buf = buf.slice(prev_len..prev_len + n);
                    buf
                });
                n
            }};
        }

        macro_rules! read_packet {
            ($reader:ident) => {{
                let (buf, n) = self.packet_recv($reader).await?;
                self.store(|| buf.clone());
                out_buf.put(buf);
                n
            }};
        }

        match &mut *self.reader.lock().await {
            ReaderType::Tcp(reader) => Ok(read_stream!(reader)),
            ReaderType::Tls(reader) => Ok(read_stream!(reader)),
            ReaderType::Quic(reader) => Ok(read_stream!(reader)),
            ReaderType::Udp(reader) => Ok(read_packet!(reader)),
            ReaderType::Unknown => unreachable!(),
        }
    }

    pub async fn read_exact(&self, len: usize) -> Result<Bytes> {
        let cache_len = self.cache_len();

        macro_rules! read_stream {
            ($reader:ident) => {{
                let mut req_buf = vec![0u8; len - cache_len];
                // dbg!(len - cache_len);
                $reader.read_exact(&mut req_buf).await?;

                let mut cache = self.cache.lock();
                cache.push_back(req_buf.into());
            }};
        }

        macro_rules! read_packet {
            ($reader:ident) => {{
                let req_buf_len = len - cache_len;
                let (packet, size) = self.packet_recv($reader).await?;

                if size < req_buf_len {
                    let msg = format!(
                        "read_exact error due to: new udp packet size = {} is less than required len = {}",
                        size, req_buf_len,
                    );
                    return Err(Error::msg(msg));
                }

                let mut cache = self.cache.lock();
                cache.push_back(packet);
            }};
        }

        // cached data is not enough
        if len > cache_len {
            match &mut *self.reader.lock().await {
                ReaderType::Tcp(reader) => read_stream!(reader),
                ReaderType::Tls(reader) => read_stream!(reader),
                ReaderType::Quic(reader) => read_stream!(reader),
                ReaderType::Udp(reader) => read_packet!(reader),
                ReaderType::Unknown => unreachable!(),
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

    async fn packet_recv(&self, socket: &Arc<UdpSocket>) -> Result<(Bytes, usize)> {
        let mut buf = vec![0u8; constants::UDP_MTU];
        let (len, _addr) = socket.recv_from(&mut buf).await?;

        if let Some(packet) = buf.get(0..len) {
            Ok((Bytes::copy_from_slice(packet), packet.len()))
        } else {
            Err(Error::msg("error recv from remote"))
        }
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
