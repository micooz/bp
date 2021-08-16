use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

/// TcpStreamReader
pub struct TcpStreamReader {
    inner: ReadHalf<TcpStream>,
    cache: BytesMut,
}

impl TcpStreamReader {
    pub fn new(read_half: ReadHalf<TcpStream>) -> Self {
        Self {
            inner: read_half,
            cache: BytesMut::with_capacity(32),
        }
    }

    pub async fn read_buf(&mut self, capacity: usize) -> crate::Result<Bytes> {
        let mut buf = BytesMut::with_capacity(capacity);
        self.read_into(&mut buf).await?;
        Ok(buf.freeze())
    }

    pub async fn read_into(&mut self, buf: &mut BytesMut) -> crate::Result<()> {
        if !self.cache.is_empty() {
            buf.put(self.cache.clone().freeze());
            self.cache.clear();
            return Ok(());
        }
        if 0 == self.inner.read_buf(buf).await? {
            return Err("read_buf return 0".into());
        }
        Ok(())
    }

    pub async fn read_exact(&mut self, len: usize) -> crate::Result<Bytes> {
        let cache_len = self.cache.len();

        let final_buf = if len > cache_len {
            let mut buf = vec![0u8; len - cache_len];
            self.inner.read_exact(&mut buf).await?;

            if cache_len > 0 {
                buf = [self.cache.to_vec(), buf].concat();
                self.cache.clear();
            }

            buf
        } else {
            let (left, _) = self.cache.split_at(len);
            let buf = left.to_vec();
            self.cache.advance(len);

            buf
        };

        Ok(final_buf.into())
    }

    pub fn cache(&mut self, buf: &Bytes) {
        self.cache.put(buf.clone());
    }
}

/// TcpStreamWriter
#[derive(Debug)]
pub struct TcpStreamWriter {
    inner: WriteHalf<TcpStream>,
}

impl TcpStreamWriter {
    pub fn new(write_half: WriteHalf<TcpStream>) -> Self {
        Self { inner: write_half }
    }

    pub async fn write(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        self.inner.write(buf).await?;
        Ok(())
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> tokio::io::Result<()> {
        self.inner.write_all(buf).await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> tokio::io::Result<()> {
        self.inner.flush().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> tokio::io::Result<()> {
        self.inner.shutdown().await?;
        Ok(())
    }
}

#[test]
fn test_buf_advance() {
    let mut buf = BytesMut::from("123");
    assert!(buf.len() == 3);

    buf.advance(1);
    assert!(buf.len() == 2);
}
