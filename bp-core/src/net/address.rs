use crate::{net::dns::lookup, net::socket, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    convert::TryInto,
    fmt::Display,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

// The same as ATYP in Socks5 Protocol
pub enum AddressType {
    V4,
    V6,
    HostName,
}

impl From<AddressType> for u8 {
    fn from(value: AddressType) -> u8 {
        match value {
            AddressType::V4 => 1,
            AddressType::V6 => 4,
            AddressType::HostName => 3,
        }
    }
}

impl TryInto<AddressType> for u8 {
    type Error = String;

    fn try_into(self) -> std::result::Result<AddressType, Self::Error> {
        match self {
            1 => Ok(AddressType::V4),
            4 => Ok(AddressType::V6),
            3 => Ok(AddressType::HostName),
            _ => Err(format!("cannot parse {} to AddressType", self)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Host {
    Ip(IpAddr),
    Name(String),
}

impl ToString for Host {
    fn to_string(&self) -> String {
        match self {
            Host::Ip(ip) => ip.to_string(),
            Host::Name(name) => name.clone(),
        }
    }
}

// +------+----------+----------+
// | ATYP | DST.ADDR | DST.PORT |
// +------+----------+----------+
// |  1   | Variable |    2     |
// +------+----------+----------+
#[derive(Debug, Clone)]
pub struct Address {
    pub host: Host,
    pub port: u16,
}

impl Address {
    pub fn new(host: Host, port: u16) -> Self {
        Self { host, port }
    }

    pub fn is_ip(&self) -> bool {
        match self.host {
            Host::Ip(_) => true,
            Host::Name(_) => false,
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}:{}", self.host.to_string(), self.port)
    }

    pub fn as_bytes(&self) -> Bytes {
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
                buf.put_u8(name.len() as u8); // length of hostname
                buf.put_slice(name.as_str().as_bytes());
            }
        }

        buf.put_u16(self.port);
        buf.freeze()
    }

    pub fn as_socket_addr(&self) -> std::net::SocketAddr {
        self.as_string().parse().unwrap()
    }

    pub async fn from_socket(socket: &socket::Socket) -> Result<Self> {
        let buf = socket.read_exact(1).await?;

        let atyp: AddressType = buf[0].try_into().map_err(|_| {
            format!(
                "ATYP must be {:#04x} or {:#04x} or {:#04x} but got {:#04x}",
                u8::from(AddressType::V4),
                u8::from(AddressType::HostName),
                u8::from(AddressType::V6),
                buf[0]
            )
        })?;

        match atyp {
            AddressType::V4 => {
                let buf = socket.read_exact(6).await?;

                let host = Host::Ip(IpAddr::V4(Ipv4Addr::from([buf[0], buf[1], buf[2], buf[3]])));
                let port = u16::from_be_bytes([buf[4], buf[5]]);

                Ok(Self::new(host, port))
            }
            AddressType::V6 => {
                let buf = socket.read_exact(18).await?;

                let host = Host::Ip(IpAddr::V6(Ipv6Addr::from([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11],
                    buf[12], buf[13], buf[14], buf[15],
                ])));
                let port = u16::from_be_bytes([buf[16], buf[17]]);

                Ok(Self::new(host, port))
            }
            AddressType::HostName => {
                let buf = socket.read_exact(1).await?;
                let len = buf[0] as usize;

                let buf = socket.read_exact(len).await?;
                let host = Host::Name(String::from_utf8(buf.to_vec())?);

                let buf = socket.read_exact(2).await?;
                let port = u16::from_be_bytes([buf[0], buf[1]]);

                Ok(Self::new(host, port))
            }
        }
    }

    pub fn from_bytes(mut buf: Bytes) -> Result<(Self, Option<Bytes>)> {
        let atyp: AddressType = buf[0].try_into().map_err(|_| {
            format!(
                "ATYP must be {:#04x} or {:#04x} or {:#04x} but got {:#04x}",
                u8::from(AddressType::V4),
                u8::from(AddressType::HostName),
                u8::from(AddressType::V6),
                buf[0]
            )
        })?;

        buf.advance(1);

        let addr = match atyp {
            AddressType::V4 => {
                let host = Host::Ip(IpAddr::V4(Ipv4Addr::from([buf[0], buf[1], buf[2], buf[3]])));
                let port = u16::from_be_bytes([buf[4], buf[5]]);

                buf.advance(4 + 2);

                Self::new(host, port)
            }
            AddressType::V6 => {
                let host = Host::Ip(IpAddr::V6(Ipv6Addr::from([
                    buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9], buf[10], buf[11],
                    buf[12], buf[13], buf[14], buf[15],
                ])));
                let port = u16::from_be_bytes([buf[16], buf[17]]);

                buf.advance(16 + 2);

                Self::new(host, port)
            }
            AddressType::HostName => {
                let len = buf[0] as usize;
                buf.advance(1);

                let host = Host::Name(String::from_utf8(buf.slice(0..len).to_vec())?);
                buf.advance(len);

                let port = u16::from_be_bytes([buf[0], buf[1]]);
                buf.advance(2);

                Self::new(host, port)
            }
        };

        Ok((addr, if !buf.is_empty() { Some(buf) } else { None }))
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    // use trust dns resolve addr
    pub async fn dns_resolve(&self) -> Vec<SocketAddr> {
        match &self.host {
            Host::Ip(ip) => {
                let addr = SocketAddr::new(*ip, self.port);
                [addr].to_vec()
            }
            Host::Name(name) => {
                let response = lookup(name).await.unwrap();
                response
                    .iter()
                    .map(|addr| SocketAddr::new(addr, self.port))
                    .collect::<Vec<SocketAddr>>()
            }
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (address_type, addr) = match &self.host {
            Host::Ip(ip) => ("Ip", ip.to_string()),
            Host::Name(name) => ("HostName", name.to_string()),
        };
        write!(f, "{}({}:{})", address_type, addr, self.port)
    }
}

impl FromStr for Address {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let v: Vec<&str> = s.split(':').collect();

        if v.len() != 2 {
            return Err("invalid format of str");
        }

        let host = v[0];
        let port: u16 = v[1].parse().map_err(|_| "cannot parse port")?;

        let addr = format!("{}:{}", host, port);

        match addr.parse::<SocketAddr>() {
            Ok(v) => Ok(Self::new(Host::Ip(v.ip()), v.port())),
            Err(_) => Ok(Self::new(Host::Name(host.into()), port)),
        }
    }
}

impl From<SocketAddr> for Address {
    fn from(addr: SocketAddr) -> Self {
        Self {
            host: Host::Ip(addr.ip()),
            port: addr.port(),
        }
    }
}