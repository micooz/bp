use super::{address::NetAddr, AcceptResult};
use tokio::{net::TcpListener, sync::mpsc};

pub async fn bootstrap(local_addr: NetAddr, sender: mpsc::Sender<AcceptResult>) {
    let listener = match TcpListener::bind(&local_addr.as_string()).await {
        Ok(value) => {
            log::info!(
                "service running at {}, waiting for connection...",
                &local_addr
            );
            value
        }
        Err(err) => {
            log::error!("service bind to {} failed due to: {}", &local_addr, err);
            return;
        }
    };

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        sender.send(AcceptResult { socket, addr }).await.unwrap();
    }
}
