use crate::net::{AcceptResult, NetAddr};
use tokio::{net::TcpListener, sync::mpsc};

pub async fn bootstrap(local_addr: NetAddr, sender: mpsc::Sender<AcceptResult>) -> std::io::Result<()> {
    let listener = TcpListener::bind(&local_addr.as_string()).await?;

    log::info!("service running at {}, waiting for connection...", &local_addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        sender.send(AcceptResult { socket, addr }).await.unwrap();
    }
}
