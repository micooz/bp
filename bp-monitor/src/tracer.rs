use std::sync::Arc;

use serde::Serialize;

use crate::{events::Event, sender::SenderType};

#[derive(Default)]
pub struct Tracer {
    senders: Vec<Arc<SenderType>>,
}

impl Tracer {
    pub async fn add_sender(&mut self, mut sender: SenderType) {
        sender.init().await;
        self.senders.push(Arc::new(sender));
    }

    pub fn log<T: Serialize + Event>(&self, event: T) {
        let data = bincode::serialize(&event).unwrap();

        // dispatch data to all senders
        for sender in &self.senders {
            let sender = sender.clone();
            let data = data.clone();

            tokio::spawn(async move {
                sender.send(&data).await.unwrap();
            });
        }
    }
}
