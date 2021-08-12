use crate::net::{TcpStreamReader, TcpStreamWriter};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

pub fn split_tcp_stream(stream: TcpStream) -> (Arc<Mutex<TcpStreamReader>>, Arc<Mutex<TcpStreamWriter>>) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = TcpStreamReader::new(read_half);
    let writer = TcpStreamWriter::new(write_half);

    let reader = Arc::new(Mutex::new(reader));
    let writer = Arc::new(Mutex::new(writer));

    (reader, writer)
}
