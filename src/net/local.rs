use crate::net::AcceptResult;
use tokio::{net::TcpListener, sync::mpsc};

pub async fn bootstrap(bind_addr: String, sender: mpsc::Sender<AcceptResult>) -> std::io::Result<()> {
    let listener = TcpListener::bind(&bind_addr).await?;

    log::info!("service running at {}, waiting for connection...", bind_addr);

    loop {
        let (socket, addr) = listener.accept().await?;
        sender.send(AcceptResult { socket, addr }).await.unwrap();
    }
}
