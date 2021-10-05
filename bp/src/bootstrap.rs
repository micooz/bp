use crate::options::Options;
use bp_lib::{
    net::{
        address::Address,
        connection::{Connection, ConnectionOptions},
        service,
    },
    SharedData,
};
use std::sync::Arc;
use tokio::{sync, task, time};

#[cfg(feature = "monitor")]
use bp_monitor::MonitorCommand;

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

    // start_dns_server().await;

    Ok(())
}

fn start_main_service(
    opts: Options,
    #[cfg(feature = "monitor")] mut rx: sync::mpsc::Receiver<MonitorCommand>,
) -> task::JoinHandle<()> {
    let mut receiver = service::start_service("main", opts.bind.parse().unwrap(), opts.enable_udp);

    let shared_data = Arc::new(sync::RwLock::new(SharedData::default()));

    #[cfg(feature = "monitor")]
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
            let local_addr = opts.bind.parse::<Address>().unwrap();
            let server_addr = if opts.server_host.is_some() && opts.server_port.is_some() {
                Some(
                    format!(
                        "{}:{}",
                        opts.server_host.as_ref().unwrap(),
                        opts.server_port.as_ref().unwrap()
                    )
                    .parse::<Address>()
                    .unwrap(),
                )
            } else {
                None
            };
            let shared_data = shared_data.clone();

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let addr = socket.peer_addr().unwrap();
                let is_tcp = socket.is_tcp();

                if is_tcp {
                    log::info!("[{}] connected", addr);
                } else {
                    log::info!("[{}] received an udp packet: {} bytes", addr, socket.cache_size().await);
                }

                let mut conn = Connection::new(
                    socket,
                    ConnectionOptions {
                        id,
                        service_type: opts.get_service_type().unwrap(),
                        protocol: opts.protocol.clone(),
                        key: opts.key.clone(),
                        local_addr: local_addr.clone(),
                        server_addr: server_addr.clone(),
                        shared_data: shared_data.clone(),
                    },
                );

                if let Err(err) = conn.handle().await {
                    log::error!("{}", err);
                    let _ = conn.force_close().await;
                }

                log::info!("[{}] closed", addr);

                drop(conn);

                // remove item from shared_data after 1min
                time::sleep(time::Duration::from_secs(60)).await;
                sync::RwLock::write(&shared_data).await.conns.remove(&id);
            });
        }
    })
}

#[cfg(feature = "monitor")]
fn start_monitor_service(opts: Options, tx: sync::mpsc::Sender<MonitorCommand>) {
    use bp_monitor::handle_conn;

    // start monitor service
    let bind_addr_monitor = opts.get_monitor_bind_addr();
    let mut receiver = service::start_service("monitor", bind_addr_monitor.parse().unwrap());

    tokio::spawn(async move {
        while let Some(socket) = receiver.recv().await {
            handle_conn(socket, tx.clone());
        }
    });
}
