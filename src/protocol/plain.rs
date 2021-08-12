use crate::{
    event::{Event, EventSender},
    net::{Address, TcpStreamReader, TcpStreamWriter},
    protocol::Protocol,
    Result,
};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};

const RECV_BUFFER_SIZE: usize = 4 * 1024;

/// # Protocol
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
pub struct Plain {
    header_sent: bool,
    proxy_address: Option<Address>,
}

impl Plain {
    pub fn new() -> Self {
        Self {
            header_sent: false,
            proxy_address: None,
        }
    }
}

impl Clone for Plain {
    fn clone(&self) -> Self {
        Self {
            header_sent: self.header_sent,
            proxy_address: self.proxy_address.clone(),
        }
    }
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)> {
        let header = Address::from_reader(reader).await?;
        Ok((header, None))
    }

    async fn client_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let mut frame = BytesMut::new();

        if !self.header_sent {
            let header = self.proxy_address.as_ref().unwrap();
            frame.put(header.as_bytes());
            self.header_sent = true;
        }

        let buf = reader.read_buf(1024).await?;
        frame.put(buf);

        tx.send(Event::EncodeDone(frame.freeze())).await?;

        Ok(())
    }

    async fn server_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()> {
        let buf = reader.read_buf(RECV_BUFFER_SIZE).await?;
        tx.send(Event::EncodeDone(buf)).await?;
        Ok(())
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
