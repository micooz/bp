use std::sync::Arc;

use tokio::net::{TcpStream, UdpSocket};

use super::{reader::SocketReader, writer::SocketWriter};

pub fn split_tcp(stream: TcpStream) -> (SocketReader, SocketWriter) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = SocketReader::from_tcp(read_half);
    let writer = SocketWriter::from_tcp(write_half);

    (reader, writer)
}

pub fn split_udp(socket: Arc<UdpSocket>) -> (SocketReader, SocketWriter) {
    let reader = SocketReader::from_udp(socket.clone());
    let writer = SocketWriter::from_udp(socket);

    (reader, writer)
}

pub fn split_quic(stream: (quinn::SendStream, quinn::RecvStream)) -> (SocketReader, SocketWriter) {
    let (send, recv) = stream;

    let reader = SocketReader::from_quic(recv);
    let writer = SocketWriter::from_quic(send);

    (reader, writer)
}
