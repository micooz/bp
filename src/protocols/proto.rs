use crate::{
    net::address::{Host, NetAddr},
    Result, TcpStreamReader, TcpStreamWriter,
};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use std::net::IpAddr;

#[async_trait]
pub trait Protocol {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr>;

    async fn pack(&self, buf: Bytes) -> Result<Bytes>;

    async fn unpack(&self, buf: Bytes) -> Result<Bytes>;
}

// The same as ATYP in Socks5 Protocol
pub enum AddressType {
    V4,
    V6,
    HostName,
}

impl Into<u8> for AddressType {
    fn into(self) -> u8 {
        match self {
            AddressType::V4 => 1u8,
            AddressType::V6 => 4u8,
            AddressType::HostName => 3u8,
        }
    }
}

// +------+----------+----------+
// | ATYP | DST.ADDR | DST.PORT |
// +------+----------+----------+
// |  1   | Variable |    2     |
// +------+----------+----------+
pub struct ProxyHeader {
    host: Host,
    port: u16,
}

impl ProxyHeader {
    pub fn new(host: Host, port: u16) -> Self {
        ProxyHeader { host, port }
    }

    pub fn encode(&self) -> Bytes {
        let mut buf = BytesMut::new();

        match &self.host {
            Host::Ip(ip) => match ip {
                IpAddr::V4(v4) => {
                    buf.put_u8(AddressType::V4.into());
                    buf.put_slice(&v4.octets()[..]);
                }
                IpAddr::V6(v6) => {
                    buf.put_u8(AddressType::V6.into());
                    buf.put_slice(&v6.octets()[..]);
                }
            },
            Host::Name(name) => {
                buf.put_u8(AddressType::HostName.into());
                buf.put_slice(name.as_str().as_bytes());
            }
        }

        buf.put_u16(self.port);
        buf.freeze()
    }
}
