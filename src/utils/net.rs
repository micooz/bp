use crate::net::{TcpStreamReader, TcpStreamWriter};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

pub fn split_tcp_stream(stream: TcpStream) -> (Arc<Mutex<TcpStreamReader>>, Arc<Mutex<TcpStreamWriter>>) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = Arc::new(Mutex::new(read_half));
    let writer = Arc::new(Mutex::new(write_half));

    (reader, writer)
}
