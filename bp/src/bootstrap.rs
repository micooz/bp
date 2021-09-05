use crate::options::Options;
use bp_lib::{
    net::{
        connection::{Connection, ConnectionOptions},
        service::start_service,
    },
    SharedData,
};
#[cfg(feature = "monitor")]
use bp_monitor::MonitorCommand;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::JoinHandle, time::sleep};

pub async fn bootstrap(opts: Options) -> std::io::Result<()> {
    #[cfg(feature = "monitor")]
    let (tx, rx) = tokio::sync::mpsc::channel::<MonitorCommand>(32);

    #[cfg(feature = "monitor")]
    start_monitor_service(opts.clone(), tx);

    start_main_service(
        opts.clone(),
        #[cfg(feature = "monitor")]
        rx,
    )
    .await?;

    Ok(())
}

fn start_main_service(
    opts: Options,
    #[cfg(feature = "monitor")] mut rx: tokio::sync::mpsc::Receiver<MonitorCommand>,
) -> JoinHandle<()> {
    let mut receiver = start_service(opts.bind.clone(), "main");

    let shared_data = Arc::new(RwLock::new(SharedData::default()));
    let shared_data_monitor = shared_data.clone();

    #[cfg(feature = "monitor")]
    tokio::spawn(async move {
        while let Some(mut cmd) = rx.recv().await {
            cmd.exec(shared_data_monitor.clone()).await;
        }
    });

    tokio::spawn(async move {
        let mut id = 0usize;

        while let Some(socket) = receiver.recv().await {
            id += 1;

            let opts = opts.clone();
            let shared_data = shared_data.clone();

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
                        shared_data: shared_data.clone(),
                    },
                );

                log::info!("[{}] connected", addr);

                if let Err(err) = conn.handle().await {
                    log::error!("{}", err);
                    let _ = conn.force_close().await;
                }

                log::info!("[{}] disconnected", addr);
                drop(conn);

                // remove item from shared_data after 1min
                sleep(Duration::from_secs(60)).await;
                shared_data.write().await.conns.remove(&id);
            });
        }
    })
}

#[cfg(feature = "monitor")]
fn start_monitor_service(opts: Options, tx: tokio::sync::mpsc::Sender<MonitorCommand>) {
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
