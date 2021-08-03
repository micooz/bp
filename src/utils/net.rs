use crate::{
    net::{TcpStreamReader, TcpStreamWriter},
    Result,
};
use bytes::{Bytes, BytesMut};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, net::TcpStream, sync::Mutex};

pub fn split_tcp_stream(stream: TcpStream) -> (Arc<Mutex<TcpStreamReader>>, Arc<Mutex<TcpStreamWriter>>) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = Arc::new(Mutex::new(read_half));
    let writer = Arc::new(Mutex::new(write_half));

    (reader, writer)
}

pub async fn read_buf(reader: &mut TcpStreamReader, capacity: usize) -> Result<Bytes> {
    let mut buf = BytesMut::with_capacity(capacity);
    if 0 == reader.read_buf(&mut buf).await? {
        return Err("read_buf return 0".into());
    }
    Ok(buf.freeze())
}

pub async fn read_exact(reader: &mut TcpStreamReader, len: usize) -> Result<Bytes> {
    let mut enc_pad_len = vec![0u8; len];
    reader.read_exact(&mut enc_pad_len).await?;
    Ok(enc_pad_len.into())
}
