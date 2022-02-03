use std::{net::SocketAddr, sync::Arc};

use serde::Serialize;
use tokio::{net::UdpSocket, sync::Mutex};

use crate::events::Event;

#[derive(Default)]
pub struct Tracer {
    subscribers: Arc<Mutex<Vec<SocketAddr>>>,
    sender: Option<Arc<UdpSocket>>,
}

impl Tracer {
    pub async fn init(&mut self) {
        let addr = format!("{}:{}", "0.0.0.0", "8080");
        let addr = addr.parse::<SocketAddr>().unwrap();
        let socket = UdpSocket::bind(addr).await.unwrap();

        self.sender = Some(Arc::new(socket));
    }

    pub async fn add_subscriber(&mut self, peer_addr: SocketAddr) {
        let mut subscribers = self.subscribers.lock().await;
        subscribers.push(peer_addr);
    }

    pub fn log<T: Serialize + Event>(&self, event: T) {
        let data = bincode::serialize(&event).unwrap();

        let sender = self.sender.clone().unwrap();
        let subscribers = self.subscribers.clone();

        tokio::spawn(async move {
            let subscribers = subscribers.lock().await;

            for sub in subscribers.iter() {
                sender.send_to(&data, sub).await.unwrap();
            }
        });
    }
}
