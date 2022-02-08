use std::str::FromStr;

use anyhow::{Error, Result};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use httparse::Request;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use url::Url;

use crate::{
    net::{
        address::{Address, Host},
        socket::Socket,
    },
    protos::{Protocol, ProtocolType, ResolvedResult},
};

#[derive(Clone)]
pub struct Http {
    resolved_result: Option<ResolvedResult>,
    basic_auth: Option<HttpBasicAuth>,
}

impl Http {
    pub fn new(basic_auth: Option<HttpBasicAuth>) -> Self {
        Self {
            resolved_result: None,
            basic_auth,
        }
    }

    fn basic_authorization_verify(req: &Request, auth: &HttpBasicAuth) -> Result<()> {
        let auth_header = req
            .headers
            .iter()
            .find(|item| item.name.to_uppercase() == "AUTHORIZATION")
            .ok_or_else(|| Error::msg("authorization required but Authorization Header is not found"))?;

        let value = String::from_utf8(auth_header.value.to_vec())?;
        let mut split = value.split(' ');

        let auth_type = split
            .next()
            .ok_or_else(|| Error::msg("invalid authorization, type is not found"))?;

        if auth_type.to_lowercase() != "basic" {
            return Err(Error::msg("invalid authorization, only support Basic Authorization"));
        }

        let auth_credentials = split
            .next()
            .ok_or_else(|| Error::msg("invalid authorization, credentials is not found"))?;

        let auth_credentials = base64::decode(auth_credentials)
            .map_err(|_| Error::msg("invalid authorization, credentials is not base64 encoded"))?;

        let auth_credentials = String::from_utf8(auth_credentials)
            .map_err(|_| Error::msg("invalid authorization, credentials is not base64 encoded utf-8 string"))?;

        let mut split = auth_credentials.split(':');

        let user = split
            .next()
            .ok_or_else(|| Error::msg("invalid authorization, user is not found"))?;

        let password = split
            .next()
            .ok_or_else(|| Error::msg("invalid authorization, password is not found"))?;

        // compare
        if auth.user != user || auth.password != password {
            return Err(Error::msg("invalid authorization, mismatch user or password"));
        }

        Ok(())
    }
}

#[async_trait]
impl Protocol for Http {
    fn get_name(&self) -> String {
        "http".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> &ResolvedResult {
        self.resolved_result.as_ref().unwrap()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<&ResolvedResult> {
        let mut buf = BytesMut::with_capacity(1024);

        loop {
            socket.read_into(&mut buf).await?;

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);

            let status = req.parse(&buf[..])?;

            // waiting request frame complete
            if !status.is_complete() {
                continue;
            }

            let path = req.path.unwrap();
            let method = req.method.unwrap();

            if method.to_uppercase() == "CONNECT" {
                // TODO: Proxy Authorization

                // for HTTP proxy tunnel requests
                let addr = Address::from_str(path).map_err(|err| Error::msg(err.to_string()))?;
                let resp = Bytes::from_static(b"HTTP/1.1 200 Connection Established\r\n\r\n");

                socket.send(&resp).await?;

                self.set_resolved_result(ResolvedResult {
                    protocol: ProtocolType::HttpProxy,
                    address: addr,
                    pending_buf: None,
                });

                return Ok(self.get_resolved_result());
            } else {
                // for direct HTTP requests

                // Basic Authorization check
                if let Some(auth) = &self.basic_auth {
                    if let Err(err) = Self::basic_authorization_verify(&req, auth) {
                        socket.send(b"HTTP/1.1 401 Unauthorized\r\n\r\n").await?;
                        return Err(err);
                    }
                }

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
                                if host.contains(':') {
                                    Address::from_str(&host).map_err(|err| Error::msg(err.to_string()))?
                                } else {
                                    Address::new(Host::Name(host), 80)
                                }
                            }
                            None => return Err(err.into()),
                        }
                    }
                };

                self.set_resolved_result(ResolvedResult {
                    protocol: ProtocolType::Http,
                    address: addr,
                    pending_buf: None,
                });

                return Ok(self.get_resolved_result());
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

#[derive(Debug, Clone)]
pub struct HttpBasicAuth {
    user: String,
    password: String,
}

impl FromStr for HttpBasicAuth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(':');
        let user = split.next().ok_or("invalid format of user")?;
        let password = split.next().ok_or("invalid format of password")?;

        if user.is_empty() || password.is_empty() {
            return Err("user or password cannot be empty".to_string());
        }

        Ok(Self {
            user: user.to_string(),
            password: password.to_string(),
        })
    }
}

impl Serialize for HttpBasicAuth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.user, self.password))
    }
}

struct HttpBasicAuthVisitor;

impl<'de> Visitor<'de> for HttpBasicAuthVisitor {
    type Value = HttpBasicAuth;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("<user>:<password>")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Self::Value::from_str(v).map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for HttpBasicAuth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(HttpBasicAuthVisitor)
    }
}
