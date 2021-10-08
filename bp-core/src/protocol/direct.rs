use crate::{
    event::{Event, EventSender},
    net::socket,
    protocol, Result,
};
use async_trait::async_trait;

const RECV_BUFFER_SIZE: usize = 4 * 1024;

#[derive(Default, Clone)]
pub struct Direct {}

#[async_trait]
impl protocol::Protocol for Direct {
    fn get_name(&self) -> String {
        "direct".into()
    }

    async fn resolve_proxy_address(&mut self, _socket: &socket::Socket) -> Result<protocol::ResolvedResult> {
        unimplemented!("direct protocol cannot be used on inbound")
    }

    async fn client_encode(&mut self, socket: &socket::Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::ClientEncodeDone(buf)).await?;
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
