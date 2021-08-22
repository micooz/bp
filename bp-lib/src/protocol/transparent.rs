use crate::{
    event::{Event, EventSender},
    net::{
        address::Address,
        io::{TcpStreamReader, TcpStreamWriter},
    },
    protocol::Protocol,
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;

const RECV_BUFFER_SIZE: usize = 4 * 1024;

pub struct Transparent {}

impl Transparent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Clone for Transparent {
    fn clone(&self) -> Self {
        Self {}
    }
}

#[async_trait]
impl Protocol for Transparent {
    fn get_name(&self) -> String {
        "transparent".into()
    }

    fn set_proxy_address(&mut self, _addr: Address) {}

    fn get_proxy_address(&self) -> Option<Address> {
        unimplemented!()
    }

    async fn resolve_proxy_address(
        &mut self,
        _reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)> {
        unimplemented!("transparent protocol cannot be used on inbound")
    }

    async fn client_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let buf = reader.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::EncodeDone(buf)).await?;
        Ok(())
    }

    async fn server_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        self.client_encode(reader, tx).await
    }

    async fn client_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let buf = reader.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::DecodeDone(buf)).await?;
        Ok(())
    }

    async fn server_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        self.client_decode(reader, tx).await
    }
}
