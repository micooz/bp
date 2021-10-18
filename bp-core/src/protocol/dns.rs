use simple_dns::{Packet, SimpleDnsError};

pub struct Dns {}

impl Dns {
    pub fn parse(buf: &[u8]) -> Result<Packet, SimpleDnsError> {
        Packet::parse(buf)
    }
}
