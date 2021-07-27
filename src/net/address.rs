use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

#[derive(Clone)]
pub enum Address {
    Ip(IpAddr),
    HostName(String),
}

impl ToString for Address {
    fn to_string(&self) -> String {
        match self {
            Address::Ip(ip) => ip.to_string(),
            Address::HostName(name) => name.clone(),
        }
    }
}

#[derive(Clone)]
pub struct NetAddr {
    address: Address,
    port: u16,
}

impl NetAddr {
    pub fn new(address: Address, port: u16) -> Self {
        NetAddr { address, port }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.address.to_string(), self.port)
    }
}

impl Display for NetAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (address_type, addr) = match &self.address {
            Address::Ip(ip) => ("Ip", ip.to_string()),
            Address::HostName(name) => ("HostName", name.to_string()),
        };
        write!(f, "{}({}:{})", address_type, addr, self.port)
    }
}

impl FromStr for NetAddr {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split(':').collect();

        if v.len() != 2 {
            return Err("invalid format of str");
        }

        let host = v[0];
        let port: u16 = v[1].parse().unwrap();

        let addr = format!("{}:{}", host, port);

        match addr.parse::<SocketAddr>() {
            Ok(v) => Ok(Self {
                address: Address::Ip(v.ip()),
                port: v.port(),
            }),
            Err(_) => Ok(Self {
                address: Address::HostName(host.into()),
                port,
            }),
        }
    }
}
