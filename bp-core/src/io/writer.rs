use std::{net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::{TcpStream, UdpSocket},
    sync::Mutex,
};

#[derive(Debug)]
enum WriterType {
    Unknown,
    Tcp(WriteHalf<TcpStream>),
    Udp(Arc<UdpSocket>),
    Quic(quinn::SendStream),
}

impl Default for WriterType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// SocketWriter
#[derive(Debug, Default)]
pub struct SocketWriter {
    writer: Mutex<WriterType>,
    peer_addr: Option<SocketAddr>,
}

impl SocketWriter {
    pub fn from_tcp(write_half: WriteHalf<TcpStream>) -> Self {
        Self {
            peer_addr: None,
            writer: Mutex::new(WriterType::Tcp(write_half)),
        }
    }

    pub fn from_udp(socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> Self {
        Self {
            peer_addr: Some(peer_addr),
            writer: Mutex::new(WriterType::Udp(socket)),
        }
    }

    pub fn from_quic(send_stream: quinn::SendStream) -> Self {
        Self {
            peer_addr: None,
            writer: Mutex::new(WriterType::Quic(send_stream)),
        }
    }

    pub async fn send(&self, buf: &[u8]) -> tokio::io::Result<()> {
        macro_rules! write_stream {
            ($writer:ident) => {{
                $writer.write_all(buf).await?;
                $writer.flush().await?;
            }};
        }

        macro_rules! write_packet {
            ($writer:ident) => {{
                $writer.send_to(buf, self.peer_addr.as_ref().unwrap()).await?;
            }};
        }

        match &mut *self.writer.lock().await {
            WriterType::Tcp(writer) => write_stream!(writer),
            WriterType::Quic(writer) => write_stream!(writer),
            WriterType::Udp(writer) => write_packet!(writer),
            WriterType::Unknown => unreachable!(),
        }

        Ok(())
    }

    pub async fn close(&self) -> tokio::io::Result<()> {
        match &mut *self.writer.lock().await {
            WriterType::Tcp(writer) => writer.shutdown().await?,
            WriterType::Quic(writer) => writer.shutdown().await?,
            WriterType::Udp(_writer) => (),
            WriterType::Unknown => unreachable!(),
        }

        Ok(())
    }
}
