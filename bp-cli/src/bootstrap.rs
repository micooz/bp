use bp_core::{global, init_dns_resolver, start_service, Connection, Options, StartupInfo};
use tokio::{sync::oneshot, task, time};

#[cfg(feature = "monitor")]
use bp_monitor::MonitorCommand;

pub async fn bootstrap(opts: Options, sender_ready: oneshot::Sender<StartupInfo>) -> std::result::Result<(), String> {
    init_dns_resolver(opts.get_dns_server().as_socket_addr())
        .await
        .map_err(|x| x.to_string())?;

    #[cfg(feature = "monitor")]
    let (tx, rx) = tokio::sync::mpsc::channel::<MonitorCommand>(32);

    #[cfg(feature = "monitor")]
    start_monitor_service(opts.clone(), tx);

    let handle = start_main_service(
        opts.clone(),
        #[cfg(feature = "monitor")]
        rx,
    )
    .await?;

    sender_ready
        .send(StartupInfo {
            bind_addr: opts.bind.clone(),
        })
        .unwrap();

    handle.await.unwrap();

    Ok(())
}

async fn start_main_service(
    opts: Options,
    #[cfg(feature = "monitor")] mut rx: sync::mpsc::Receiver<MonitorCommand>,
) -> std::result::Result<task::JoinHandle<()>, String> {
    let mut receiver = start_service("main", &opts.bind).await?;

    #[cfg(feature = "monitor")]
    tokio::spawn(async move {
        while let Some(mut cmd) = rx.recv().await {
            cmd.exec(shared_data_monitor.clone()).await;
        }
    });

    let opts_for_acl = opts.clone();

    // load acl
    tokio::spawn(async move {
        if let Some(ref path) = opts_for_acl.proxy_white_list {
            let acl = global::SHARED_DATA.get_acl();

            if let Err(err) = acl.load_from_file(path.clone()) {
                log::error!("[acl] load white list failed due to: {}", err);
                return;
            }

            #[cfg(not(debug_assertions))]
            {
                let path = path.clone();

                tokio::spawn(async move {
                    acl.watch(path).unwrap();
                });
            }
        }
    });

    let handle = tokio::spawn(async move {
        let mut id = 0usize;

        while let Some(socket) = receiver.recv().await {
            id += 1;

            let opts = opts.clone();

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let peer_addr = socket.peer_addr().unwrap();

                log::info!("[{}] connected", peer_addr);

                let mut conn = Connection::new(id, socket, opts);

                if let Err(err) = conn.handle().await {
                    log::trace!("{}", err);
                    let _ = conn.close().await;
                }

                log::info!("[{}] closed", peer_addr);

                drop(conn);

                // remove item from shared_data after 1min
                time::sleep(time::Duration::from_secs(60)).await;

                global::SHARED_DATA.remove_connection_snapshot(id).await;
            });
        }
    });

    Ok(handle)
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
