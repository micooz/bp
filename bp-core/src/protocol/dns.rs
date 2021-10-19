use simple_dns::{Packet, SimpleDnsError};

pub struct Dns {}

impl Dns {
    pub fn parse(buf: &[u8]) -> Result<Packet, SimpleDnsError> {
        Packet::parse(buf)
    }
}

pub fn check_dns_query(buf: &[u8]) -> bool {
    Dns::parse(buf).is_ok()
}
