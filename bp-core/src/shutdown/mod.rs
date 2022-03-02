use tokio::sync::broadcast::{channel, Sender};

pub struct Shutdown {
    sender: Sender<()>,
}

impl Shutdown {
    pub fn new() -> Shutdown {
        let (sender, _receiver) = channel::<()>(16);
        Shutdown { sender }
    }

    pub fn broadcast(&self) -> usize {
        self.sender.send(()).unwrap()
    }

    pub async fn recv(&self) {
        let mut receiver = self.sender.subscribe();
        receiver.recv().await.unwrap();
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Shutdown {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}
