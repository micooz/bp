use crate::{
    event::EventSender,
    net::{Address, Host, TcpStreamReader, TcpStreamWriter},
    protocol::Protocol,
    Result,
};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use url::Url;

pub struct Http {
    header_sent: bool,
    proxy_address: Option<Address>,
}

impl Http {
    pub fn new() -> Self {
        Self {
            header_sent: false,
            proxy_address: None,
        }
    }
}

impl Clone for Http {
    fn clone(&self) -> Self {
        Self {
            header_sent: self.header_sent,
            proxy_address: self.proxy_address.clone(),
        }
    }
}

#[async_trait]
impl Protocol for Http {
    fn get_name(&self) -> String {
        "http".into()
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
        let mut buf = BytesMut::with_capacity(1024);
        loop {
            reader.read_into(&mut buf).await?;

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);

            let bytes = String::from_utf8(buf.to_vec())?;
            let status = req.parse(bytes.as_bytes())?;

            if status.is_complete() {
                // get url
                let path = req.path.unwrap();

                // parse url
                let parse_result = Url::parse(path)?;
                let host = parse_result.host().unwrap().to_string();
                let port = parse_result.port().unwrap_or(80);

                return Ok((Address::new(Host::Name(host), port), Some(buf.into())));
            }
        }
    }

    async fn client_encode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }
}
