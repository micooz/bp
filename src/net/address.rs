use std::{
    fmt::Display,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

#[derive(Clone)]
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

#[derive(Clone)]
pub struct NetAddr {
    pub host: Host,
    pub port: u16,
}

impl NetAddr {
    pub fn new(host: Host, port: u16) -> Self {
        NetAddr { host, port }
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}", self.host.to_string(), self.port)
    }
}

impl Display for NetAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (address_type, addr) = match &self.host {
            Host::Ip(ip) => ("Ip", ip.to_string()),
            Host::Name(name) => ("HostName", name.to_string()),
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
                host: Host::Ip(v.ip()),
                port: v.port(),
            }),
            Err(_) => Ok(Self {
                host: Host::Name(host.into()),
                port,
            }),
        }
    }
}
