use std::{net::SocketAddr, sync::Arc};

use tokio::net::UdpSocket;

pub enum Subscriber {
    Unknown,
    Udp((Arc<UdpSocket>, SocketAddr)),
}

impl PartialEq for Subscriber {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Udp((_, laddr)), Self::Udp((_, raddr))) => laddr == raddr,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
