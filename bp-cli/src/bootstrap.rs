use crate::Options;
use bp_core::{
    global,
    net::{
        self,
        service::{start_service, StartupInfo},
        Address,
    },
};
use tokio::{sync::oneshot, task, time};

#[cfg(feature = "monitor")]
use bp_monitor::MonitorCommand;

pub async fn bootstrap(opts: Options, sender: oneshot::Sender<StartupInfo>) -> std::io::Result<()> {
    #[cfg(feature = "monitor")]
    let (tx, rx) = tokio::sync::mpsc::channel::<MonitorCommand>(32);

    #[cfg(feature = "monitor")]
    start_monitor_service(opts.clone(), tx);

    start_main_service(
        opts.clone(),
        sender,
        #[cfg(feature = "monitor")]
        rx,
    )
    .await?;

    // start_dns_server().await;

    Ok(())
}

fn start_main_service(
    opts: Options,
    sender_ready: oneshot::Sender<StartupInfo>,
    #[cfg(feature = "monitor")] mut rx: sync::mpsc::Receiver<MonitorCommand>,
) -> task::JoinHandle<()> {
    let mut receiver = start_service("main", opts.bind.parse().unwrap(), opts.enable_udp, sender_ready);

    #[cfg(feature = "monitor")]
    tokio::spawn(async move {
        while let Some(mut cmd) = rx.recv().await {
            cmd.exec(shared_data_monitor.clone()).await;
        }
    });

    let opts_for_acl = opts.clone();

    // load acl
    tokio::spawn(async move {
        if let Some(ref path) = opts_for_acl.proxy_list_path {
            let acl = global::SHARED_DATA.get_acl();

            if let Err(err) = acl.load_from_file(path.clone()) {
                log::error!("[acl] load white list failed due to: {}", err);
                return;
            }

            let path = path.clone();

            tokio::spawn(async move {
                acl.watch(path).unwrap();
            });
        }
    });

    tokio::spawn(async move {
        let mut id = 0usize;

        while let Some(socket) = receiver.recv().await {
            id += 1;

            let opts = opts.clone();
            let local_addr = opts.bind.parse::<net::Address>().unwrap();
            let server_addr: Option<Address> = opts.server_bind.as_ref().map(|addr| addr.parse().unwrap());

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let addr = socket.peer_addr().unwrap();

                log::info!("[{}] connected", addr);

                let opts = net::ConnOptions {
                    id,
                    service_type: opts.get_service_type().unwrap(),
                    protocol: opts.protocol.clone(),
                    key: opts.key.clone(),
                    local_addr: local_addr.clone(),
                    server_addr: server_addr.clone(),
                    enable_white_list: opts.proxy_list_path.is_some(),
                };

                let mut conn = net::Connection::new(socket, opts);

                if let Err(err) = conn.handle().await {
                    log::error!("{}", err);
                    let _ = conn.force_close().await;
                }

                log::info!("[{}] closed", addr);

                drop(conn);

                // remove item from shared_data after 1min
                time::sleep(time::Duration::from_secs(60)).await;

                global::SHARED_DATA.remove_connection_snapshot(id).await;
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
