use std::fmt::Display;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;

pub fn start_service<A>(bind_addr: A, name: &'static str) -> mpsc::Receiver<TcpStream>
where
    A: ToSocketAddrs + Display + Send + Sync + 'static,
{
    let (tx, rx) = mpsc::channel::<TcpStream>(32);

    tokio::spawn(async move {
        if let Err(err) = listen(bind_addr, name, tx).await {
            log::error!("[{}] service start failed due to: {}", name, err);
        }
    });

    rx
}

async fn listen<A>(bind_addr: A, name: &'static str, sender: mpsc::Sender<TcpStream>) -> std::io::Result<()>
where
    A: ToSocketAddrs + Display,
{
    let listener = TcpListener::bind(&bind_addr).await?;

    log::info!("[{}] service running at {}, waiting for connection...", name, bind_addr);

    loop {
        let (socket, _) = listener.accept().await?;
        sender.send(socket).await.unwrap();
    }
}
