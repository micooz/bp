use crate::{
    event,
    net::{self, address, socket},
    protocol, Result,
};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Default)]
pub struct Http {
    header_sent: bool,
    proxy_address: Option<net::Address>,
}

#[async_trait]
impl protocol::Protocol for Http {
    fn get_name(&self) -> String {
        "http".into()
    }

    fn set_proxy_address(&mut self, addr: net::Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<net::Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<protocol::ResolvedResult> {
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
                let addr = net::Address::from_str(path)?;
                let resp = Bytes::from_static(b"HTTP/1.1 200 Connection Established\r\n\r\n");

                socket.send(&resp).await?;

                return Ok(protocol::ResolvedResult {
                    protocol: self.get_name(),
                    address: addr,
                    pending_buf: None,
                });
            } else {
                // for direct HTTP requests
                let (host, port) = match Url::parse(path) {
                    Ok(v) => {
                        let host = v.host().unwrap().to_string();
                        let port = v.port().unwrap_or(80);

                        (host, port)
                    }
                    Err(err) => {
                        // fallback to read Host header
                        let host_header = headers.iter().find(|&item| item.name.to_lowercase() == "host");
                        
                        match host_header {
                            Some(v) => {
                                let host = String::from_utf8(v.value.to_vec()).unwrap();
                                let port = 80u16;

                                (host, port)
                            },
                            None => return Err(Box::new(err)),
                        }
                    }
                };

                return Ok(protocol::ResolvedResult {
                    protocol: self.get_name(),
                    address: net::Address::new(address::Host::Name(host), port),
                    pending_buf: Some(buf.into()),
                });
            }
        }
    }

    async fn client_encode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }
}
