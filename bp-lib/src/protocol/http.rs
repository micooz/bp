use crate::{
    event::EventSender,
    net::{
        address::{Address, Host},
        socket,
    },
    protocol::Protocol,
    Result,
};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Default)]
pub struct Http {
    header_sent: bool,
    proxy_address: Option<Address>,
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

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<(Address, Option<Bytes>)> {
        let mut buf = BytesMut::with_capacity(1024);

        loop {
            socket.read_into(&mut buf).await?;

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);

            let bytes = String::from_utf8(buf.to_vec())?;
            let status = req.parse(bytes.as_bytes())?;

            if !status.is_complete() {
                continue;
            }

            let path = req.path.unwrap();
            let method = req.method.unwrap();

            if method.to_uppercase() == "CONNECT" {
                // for HTTP proxy tunnel requests
                let addr = Address::from_str(path)?;
                let resp = Bytes::from_static(b"HTTP/1.1 200 Connection Established\r\n\r\n");

                socket.send(&resp).await?;

                return Ok((addr, None));
            } else {
                // for direct HTTP requests
                let parse_result = Url::parse(path)?;

                let host = parse_result.host().unwrap().to_string();
                let port = parse_result.port().unwrap_or(80);

                return Ok((Address::new(Host::Name(host), port), Some(buf.into())));
            }
        }
    }

    async fn client_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }
}
