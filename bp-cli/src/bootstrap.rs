use std::{env, fmt::Display, sync::Arc};

use anyhow::{Error, Result};
use bp_core::{
    get_acl, init_dns_resolver, init_quic_endpoint_pool, init_quinn_client_config, init_quinn_server_config,
    init_tls_client_config, init_tls_server_config, Connection, Options, QuicService, Service, Socket, StartupInfo,
    TcpService, TlsService, UdpService,
};
use parking_lot::Mutex;
use tokio::{
    sync::{mpsc::channel, oneshot::Sender},
    task::JoinHandle,
};

#[cfg(target_family = "unix")]
use crate::daemonize::daemonize;
use crate::dirs::Dirs;

#[allow(dead_code)]
const ENV_DISABLE_DAEMONIZE: &str = "DISABLE_DAEMONIZE";
const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

pub async fn bootstrap(opts: Options, sender_ready: Sender<StartupInfo>) -> Result<()> {
    // dirs init
    Dirs::init()?;

    log::info!("log files are stored at {}", Dirs::log_file().to_str().unwrap());

    // daemonize
    // #[cfg(target_family = "unix")]
    // if opts.daemonize && !env::vars().any(|(k, _)| k == ENV_DISABLE_DAEMONIZE) {
    //     daemonize_self()?;
    //     return Ok(());
    // }

    // dns server
    init_dns_resolver(opts.dns_server().as_socket_addr()).await?;

    // init tls configs
    if opts.tls() || opts.quic() {
        init_tls_configs(&opts)?;
    }

    // init quic endpoint pool
    if opts.is_client() && opts.quic() {
        let quic_max_concurrency = opts.quic_max_concurrency().unwrap_or(u16::MAX);
        init_quic_endpoint_pool(quic_max_concurrency);
    }

    // main service
    let handle = start_main_service(opts.clone(), sender_ready).await?;

    handle.await.unwrap();

    Ok(())
}

async fn start_main_service(opts: Options, sender_ready: Sender<StartupInfo>) -> Result<JoinHandle<()>> {
    let (sender, mut receiver) = channel::<Option<Socket>>(SERVICE_CONNECTION_THRESHOLD);

    let bind_addr = opts.bind().resolve().await?;
    let bind_ip = bind_addr.ip().to_string();
    let bind_host = opts.bind().host();
    let bind_port = opts.bind().port();

    // server side enable --quic, start Quic service
    #[allow(clippy::never_loop)]
    loop {
        if opts.is_server() {
            if opts.tls() {
                TlsService::start("main", bind_addr, sender.clone()).await?;
                UdpService::start("main", bind_addr, sender).await?;
                break;
            }
            if opts.quic() {
                QuicService::start("main", bind_addr, sender).await?;
                break;
            }
        }
        TcpService::start("main", bind_addr, sender.clone()).await?;
        UdpService::start("main", bind_addr, sender).await?;
        break;
    }

    let opts_for_acl = opts.clone();

    // load acl
    tokio::spawn(async move {
        if !opts_for_acl.is_client() {
            return;
        }
        if let Some(ref path) = opts_for_acl.proxy_white_list() {
            let acl = get_acl();

            if let Err(err) = acl.load_from_file(path) {
                log::error!("[acl] load white list failed due to: {}", err);
                return;
            }

            #[cfg(not(debug_assertions))]
            {
                let path = path.clone();

                tokio::spawn(async move {
                    acl.watch(&path).unwrap();
                });
            }
        }
    });

    let handle = tokio::spawn(async move {
        let mut total_cnt = 0usize;
        let live_cnt = Arc::new(Mutex::new(Counter::default()));

        while let Some(socket) = receiver.recv().await {
            if socket.is_none() {
                break;
            }

            total_cnt += 1;

            let live_cnt = live_cnt.clone();
            live_cnt.lock().inc();

            let socket = socket.unwrap();
            let opts = opts.clone();

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let peer_addr = socket.peer_addr();

                log::info!(
                    "[{}] connected, {} live connections, {} in total",
                    peer_addr,
                    live_cnt.lock(),
                    total_cnt
                );

                let mut conn = Connection::new(socket, opts);

                if let Err(err) = conn.handle().await {
                    log::trace!("{}", err);
                    let _ = conn.close().await;
                }

                live_cnt.lock().dec();

                log::info!(
                    "[{}] closed, {} live connections, {} in total",
                    peer_addr,
                    live_cnt.lock(),
                    total_cnt
                );
            });
        }
    });

    sender_ready
        .send(StartupInfo {
            bind_addr,
            bind_ip,
            bind_host,
            bind_port,
        })
        .unwrap();

    Ok(handle)
}

fn init_tls_configs(opts: &Options) -> Result<()> {
    if opts.is_server() {
        if let (Some(cert), Some(key)) = (opts.tls_cert().as_ref(), opts.tls_key().as_ref()) {
            log::info!("loading TLS certificate from {}", cert);
            log::info!("loading TLS private key from {}", key);

            if opts.tls() {
                init_tls_server_config(cert, key)?;
            }
            if opts.quic() {
                init_quinn_server_config(cert, key)?;
            }
        }
    }

    if opts.is_client() {
        if let Some(cert) = opts.tls_cert().as_ref() {
            log::info!("loading TLS certificate from {}", cert);

            if opts.tls() {
                init_tls_client_config(cert)?;
            }
            if opts.quic() {
                init_quinn_client_config(cert)?;
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
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

    let mut command = std::process::Command::new(bin_path);
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
