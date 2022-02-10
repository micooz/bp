use std::{net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use tokio::net::UdpSocket;

use super::Sender;

#[derive(Default)]
pub struct UdpSender {
    socket: Option<Arc<UdpSocket>>,
    subscribers: Vec<SocketAddr>,
}

#[async_trait]
impl Sender for UdpSender {
    async fn init(&mut self) {
        let addr = format!("{}:{}", "0.0.0.0", "8080");
        let addr = addr.parse::<SocketAddr>().unwrap();
        self.socket = Some(Arc::new(UdpSocket::bind(addr).await.unwrap()));
    }

    fn subscribe(&mut self, peer_addr: SocketAddr) {
        self.subscribers.push(peer_addr);
    }

    async fn send(&self, buf: &[u8]) -> anyhow::Result<()> {
        let socket = self.socket.as_ref().unwrap();

        // dispatch to all subscribers
        for sub in &self.subscribers {
            let buf = buf.to_vec();
            let socket = socket.clone();
            let target = sub.clone();

            tokio::spawn(async move {
                socket.send_to(&buf, target).await.unwrap();
            });
        }

        Ok(())
    }
}
