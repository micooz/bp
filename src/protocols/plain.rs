use crate::{
    net::{NetAddr, TcpStreamReader, TcpStreamWriter},
    protocols::{DecodeStatus, Protocol},
    Result,
};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};

/// # Protocol
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
pub struct Plain {
    header_sent: bool,

    proxy_address: Option<NetAddr>,
}

impl Plain {
    pub fn new() -> Self {
        Self {
            header_sent: false,
            proxy_address: None,
        }
    }
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    fn set_proxy_address(&mut self, addr: NetAddr) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<NetAddr> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<(NetAddr, Option<Bytes>)> {
        let header = NetAddr::from_reader(reader).await?;
        Ok((header, None))
    }

    fn client_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        let mut frame = BytesMut::new();

        if !self.header_sent {
            let header = self.proxy_address.as_ref().unwrap();
            frame.put(header.as_bytes());
            self.header_sent = true;
        }

        frame.put(buf);

        Ok(frame.freeze())
    }

    fn client_decode(&mut self, buf: Bytes) -> Result<DecodeStatus> {
        Ok(DecodeStatus::Fulfil(buf))
    }

    fn server_encode(&mut self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    fn server_decode(&mut self, buf: Bytes) -> Result<DecodeStatus> {
        Ok(DecodeStatus::Fulfil(buf))
    }
}
