use crate::options::Options;
use bp_lib::net::{
    connection::{Connection, ConnectionOptions},
    service::start_service,
};
use bp_monitor::Command;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;

pub async fn bootstrap(opts: Options) -> std::io::Result<()> {
    let (tx, rx) = mpsc::channel::<Command>(32);

    #[cfg(feature = "monitor")]
    start_monitor_service(opts.clone(), tx);

    start_main_service(opts.clone(), rx).await?;

    Ok(())
}

fn start_main_service(opts: Options, mut rx: Receiver<Command>) -> JoinHandle<()> {
    let mut receiver = start_service(opts.bind.clone(), "main");

    tokio::spawn(async move {
        while let Some(mut cmd) = rx.recv().await {
            // TODO: pass shared data into exec()
            cmd.exec().await;
        }
    });

    tokio::spawn(async move {
        let mut id = 0usize;

        while let Some(socket) = receiver.recv().await {
            let opts = opts.clone();
            id += 1;

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let addr = socket.peer_addr().unwrap();

                let mut conn = Connection::new(
                    socket,
                    ConnectionOptions {
                        id,
                        service_type: opts.get_service_type().unwrap(),
                        protocol: opts.protocol.clone(),
                        key: opts.key.clone(),
                        server_host: opts.server_host.clone(),
                        server_port: opts.server_port,
                    },
                );

                log::info!("[{}] connected", addr);

                if let Err(err) = conn.handle().await {
                    log::error!("{}", err);
                    let _ = conn.force_close().await;
                }

                log::info!("[{}] disconnected", addr);
            });
        }
    })
}

#[cfg(feature = "monitor")]
fn start_monitor_service(opts: Options, tx: mpsc::Sender<Command>) {
    use bp_monitor::handle_conn;

    // start monitor service
    let bind_addr_monitor = opts.get_monitor_bind_addr();
    let mut receiver = start_service(bind_addr_monitor, "monitor");

    tokio::spawn(async move {
        while let Some(socket) = receiver.recv().await {
            handle_conn(socket, tx.clone());
        }
    });
}
