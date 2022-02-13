use std::sync::Arc;

use anyhow::{Error, Result};
use serde::Serialize;

use crate::{events::Event, subscriber::Subscriber};

#[derive(Default)]
pub struct Monitor {
    subscribers: Vec<Arc<Subscriber>>,
}

impl Monitor {
    pub fn add_subscriber(&mut self, subscriber: Subscriber) -> Result<()> {
        if self.subscribers.iter().find(|&item| &**item == &subscriber).is_some() {
            return Err(Error::msg("the subscriber is already added"));
        }
        self.subscribers.push(Arc::new(subscriber));
        Ok(())
    }

    pub fn log<T: Serialize + Event>(&self, event: T) {
        // let data = bincode::serialize(&event).unwrap();
        let data = serde_json::to_string(&event).unwrap();

        // dispatch data to all senders
        for subscriber in self.subscribers.iter() {
            let subscriber = subscriber.clone();
            let data = data.clone();

            tokio::spawn(async move {
                Monitor::send(subscriber, data.as_bytes()).await;
            });
        }
    }

    async fn send(subscriber: Arc<Subscriber>, data: &[u8]) {
        match &*subscriber {
            Subscriber::Udp((socket, peer_addr)) => {
                socket.send_to(data, peer_addr).await.unwrap();
            }
            Subscriber::Unknown => todo!(),
        }
    }
}
