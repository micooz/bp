use crate::{
    event::{Event, EventSender},
    net::{address::Address, socket},
    protocol, Result,
};
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};

const RECV_BUFFER_SIZE: usize = 4 * 1024;

/// # Protocol
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
#[derive(Clone, Default)]
pub struct Plain {
    header_sent: bool,
    proxy_address: Option<Address>,
}

#[async_trait]
impl protocol::Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<protocol::ResolvedResult> {
        let address = Address::from_socket(socket).await?;

        Ok(protocol::ResolvedResult {
            protocol: self.get_name(),
            address,
            pending_buf: None,
        })
    }

    async fn client_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()> {
        let mut frame = BytesMut::new();

        if !self.header_sent {
            let header = self.proxy_address.as_ref().unwrap();
            frame.put(header.as_bytes());
            self.header_sent = true;
        }

        let buf = socket.read_buf(1024).await?;
        frame.put(buf);

        tx.send(Event::ClientEncodeDone(frame.freeze())).await?;

        Ok(())
    }

    async fn server_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::ServerEncodeDone(buf)).await?;
        Ok(())
    }

    async fn client_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::ClientDecodeDone(buf)).await?;
        Ok(())
    }

    async fn server_decode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::ServerDecodeDone(buf)).await?;
        Ok(())
    }
}
