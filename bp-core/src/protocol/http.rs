use crate::{
    net::{
        address::{Address, Host},
        socket::Socket,
    },
    protocol::{Protocol, ProtocolType, ResolvedResult},
    Result,
};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use std::str::FromStr;
use url::Url;

#[derive(Clone, Default)]
pub struct Http {
    resolved_result: Option<ResolvedResult>,
}

#[async_trait]
impl Protocol for Http {
    fn get_name(&self) -> String {
        "http".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> Option<ResolvedResult> {
        self.resolved_result.clone()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<()> {
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

                self.set_resolved_result(ResolvedResult {
                    protocol: ProtocolType::Http,
                    address: addr,
                    pending_buf: None,
                });

                return Ok(());
            } else {
                // for direct HTTP requests
                let addr = match Url::parse(path) {
                    Ok(v) => {
                        let host = v.host().unwrap().to_string();
                        let port = v.port().unwrap_or(80);

                        Address::new(Host::Name(host), port)
                    }
                    Err(err) => {
                        // fallback to read Host header
                        let host_header = headers.iter().find(|&item| item.name.to_uppercase() == "HOST");

                        match host_header {
                            Some(v) => {
                                let host = String::from_utf8(v.value.to_vec()).unwrap();

                                // Host header maybe <host:port>
                                if host.contains(":") {
                                    Address::from_str(&host)?
                                } else {
                                    Address::new(Host::Name(host), 80)
                                }
                            }
                            None => return Err(Box::new(err)),
                        }
                    }
                };

                self.set_resolved_result(ResolvedResult {
                    protocol: ProtocolType::Http,
                    address: addr,
                    pending_buf: Some(buf.freeze()),
                });

                return Ok(());
            }
        }
    }

    async fn client_encode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }
}
