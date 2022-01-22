use std::{env, fmt::Display, process::Command, sync::Arc};

use anyhow::{Error, Result};
use bp_core::{
    global::GLOBAL_DATA, init_dns_resolver, init_quinn_client_config, init_quinn_server_config, Connection, Options,
    QuicService, Service, Socket, StartupInfo, TcpService, UdpService,
};
use parking_lot::Mutex;
use tokio::{
    sync::{mpsc::channel, oneshot::Sender},
    task::JoinHandle,
    time,
};

#[cfg(target_family = "unix")]
use crate::daemonize::daemonize;
use crate::dirs::Dirs;

const ENV_DISABLE_DAEMONIZE: &str = "DISABLE_DAEMONIZE";
const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

pub async fn bootstrap(opts: Options, sender_ready: Sender<StartupInfo>) -> Result<()> {
    // dirs init
    Dirs::init()?;

    log::info!("log files are stored at {}", Dirs::log_file().to_str().unwrap());

    // daemonize
    #[cfg(target_family = "unix")]
    if opts.daemonize && !env::vars().any(|(k, _)| k == ENV_DISABLE_DAEMONIZE) {
        daemonize_self()?;
        return Ok(());
    }

    // dns server
    init_dns_resolver(opts.get_dns_server().as_socket_addr()).await?;

    // init quinn configs
    if opts.quic {
        if opts.server {
            if let (Some(cert), Some(privatekey)) = (opts.certificate.as_ref(), opts.privatekey.as_ref()) {
                log::info!("loading certificate from {}", cert);
                log::info!("loading private key from {}", privatekey);
                init_quinn_server_config(cert, privatekey).await?;
            }
        }
        if opts.client {
            if let Some(cert) = opts.certificate.as_ref() {
                log::info!("loading certificate from {}", cert);
                init_quinn_client_config(cert).await?;
            }
        }
    }

    // monitor service
    // #[cfg(feature = "monitor")]
    // {
    //     use bp_monitor::MonitorCommand;

    //     let (tx, rx) = tokio::sync::mpsc::channel::<MonitorCommand>(32);

    //     #[cfg(feature = "monitor")]
    //     start_monitor_service(opts.clone(), tx);

    //     tokio::spawn(async move {
    //         while let Some(mut cmd) = rx.recv().await {
    //             cmd.exec(shared_data_monitor.clone()).await;
    //         }
    //     });
    // }

    // main service
    let handle = start_main_service(opts.clone()).await?;

    let startup_info = StartupInfo {
        bind_addr: opts.bind.clone(),
    };
    sender_ready.send(startup_info).unwrap();

    handle.await.unwrap();

    Ok(())
}

async fn start_main_service(opts: Options) -> Result<JoinHandle<()>> {
    let (sender, mut receiver) = channel::<Option<Socket>>(SERVICE_CONNECTION_THRESHOLD);

    // server side enable --quic, start Quic service
    if opts.quic && opts.server {
        QuicService::start("main", &opts.bind, sender.clone()).await?;
    } else {
        TcpService::start("main", &opts.bind, sender.clone()).await?;
        UdpService::start("main", &opts.bind, sender).await?;
    }

    let opts_for_acl = opts.clone();

    // load acl
    tokio::spawn(async move {
        if let Some(ref path) = opts_for_acl.proxy_white_list {
            let acl = GLOBAL_DATA.get_acl();

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
        let cnt = Arc::new(Mutex::new(Counter::default()));

        while let Some(socket) = receiver.recv().await {
            if socket.is_none() {
                break;
            }

            let cnt = cnt.clone();
            cnt.lock().inc();

            let socket = socket.unwrap();
            let opts = opts.clone();

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let peer_addr = socket.peer_addr().unwrap();

                log::info!("[{}] connected, {} live connections", peer_addr, cnt.lock());

                let mut conn = Connection::new(0, socket, opts);

                if let Err(err) = conn.handle().await {
                    log::trace!("{}", err);
                    let _ = conn.close().await;
                }

                cnt.lock().dec();

                log::info!("[{}] closed, {} live connections", peer_addr, cnt.lock());

                drop(conn);

                // remove item from shared_data after 1min
                time::sleep(time::Duration::from_secs(60)).await;

                // global::SHARED_DATA.remove_connection_snapshot(id).await;
            });
        }
    });

    Ok(handle)
}

// #[cfg(feature = "monitor")]
// fn start_monitor_service(opts: Options, tx: sync::mpsc::Sender<bp_monitor::MonitorCommand>) {
//     use bp_monitor::handle_conn;

//     // start monitor service
//     let bind_addr_monitor = opts.get_monitor_bind_addr();
//     let mut receiver = service::start_service("monitor", bind_addr_monitor.parse().unwrap());

//     tokio::spawn(async move {
//         while let Some(socket) = receiver.recv().await {
//             handle_conn(socket, tx.clone());
//         }
//     });
// }

fn daemonize_self() -> Result<()> {
    log::info!(
        "start daemonize, stdout/stderr will be redirected to {}",
        Dirs::run().to_str().unwrap()
    );

    // NOTE: must read before daemonize() call
    let bin_path = env::current_exe().unwrap();
    let work_dir = env::current_dir().unwrap();

    daemonize().map_err(|err| Error::msg(format!("fail to daemonize due to: {}", err)))?;

    log::info!("spawning a new child process before exit");

    let mut command = Command::new(bin_path);
    command.current_dir(work_dir);
    command.env(ENV_DISABLE_DAEMONIZE, "1");

    for (index, arg) in env::args().enumerate() {
        if index == 0 {
            continue;
        }
        command.arg(arg);
    }

    command.spawn()?;

    Ok(())
}

#[derive(Default)]
struct Counter {
    inner: usize,
}

impl Counter {
    pub fn inc(&mut self) {
        self.inner += 1;
    }
    pub fn dec(&mut self) {
        self.inner -= 1;
    }
}

impl Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
