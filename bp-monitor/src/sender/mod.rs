use std::net::SocketAddr;

use anyhow::Result;
use async_trait::async_trait;

mod udp;

pub use udp::UdpSender;

#[async_trait]
pub trait Sender {
    async fn init(&mut self);

    fn subscribe(&mut self, peer_addr: SocketAddr);

    async fn send(&self, buf: &[u8]) -> Result<()>;
}

pub type SenderType = Box<dyn Sender + Send + Sync + 'static>;
