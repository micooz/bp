use std::{env, future::Future, sync::Arc};

use anyhow::{Error, Result};
use bp_core::{
    acl::get_acl, init_dns_resolver, init_quic_endpoint_pool, init_quinn_client_config, init_quinn_server_config,
    init_tls_client_config, init_tls_server_config, monitor_log, set_monitor, start_monitor_service, start_pac_service,
    start_quic_service, start_tcp_service, start_tls_service, start_udp_service, Connection, Options, ServiceInfo,
    Socket, Startup,
};
use bp_monitor::{events, Monitor};
use tokio::sync::{mpsc, mpsc::Sender};

#[cfg(target_family = "unix")]
use crate::utils::daemonize::daemonize;
use crate::{dirs::Dirs, utils::counter::Counter};

pub async fn run(mut opts: Options, startup: Sender<Startup>, shutdown: impl Future) -> Result<()> {
    // try load bp service options from --config
    if let Some(config) = opts.config() {
        log::info!("loading configuration from {}", config);

        opts.try_load_from_file(&config)
            .map_err(|err| Error::msg(format!("unrecognized format of --config: {}", err)))?;
    }

    // check options
    opts.check()?;

    // bootstrap bp service
    tokio::select! {
        res = boot(opts, startup.clone()) => {
            if let Err(err) = res {
                log::error!("{}", err);
                startup.send(Startup::Fail(err)).await.unwrap();
            }
        }
        _ = shutdown => {
            log::info!("shutting down...");
        }
    };

    Ok(())
}

#[allow(dead_code)]
const ENV_DISABLE_DAEMONIZE: &str = "DISABLE_DAEMONIZE";
const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

async fn boot(opts: Options, startup: Sender<Startup>) -> Result<()> {
    // pre boot
    pre_boot(&opts).await?;
    // start services
    start_services(opts.clone(), startup).await?;

    Ok(())
}

async fn pre_boot(opts: &Options) -> Result<()> {
    // dirs init
    Dirs::init()?;

    log::info!("log files are stored at logs/bp.log");

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
        init_tls_configs(opts)?;
    }

    // init quic endpoint pool
    if opts.is_client() && opts.quic() {
        let quic_max_concurrency = opts.client_opts().quic_max_concurrency;
        init_quic_endpoint_pool(quic_max_concurrency)?;
    }

    Ok(())
}

async fn start_services(opts: Options, startup: Sender<Startup>) -> Result<()> {
    let (sender, mut receiver) = mpsc::channel::<Option<Socket>>(SERVICE_CONNECTION_THRESHOLD);

    let bind_addr = opts.bind().resolve().await?;
    let bind_ip = bind_addr.ip().to_string();
    let bind_host = opts.bind().host();
    let bind_port = opts.bind().port();

    if opts.is_server() {
        #[allow(clippy::never_loop)]
        loop {
            // server side enable --tls, start TLS service
            if opts.tls() {
                start_tls_service(bind_addr, sender.clone()).await?;
                start_udp_service(bind_addr, sender.clone()).await?;
                break;
            }
            // server side enable --quic, start QUIC service
            if opts.quic() {
                start_quic_service(bind_addr, sender.clone()).await?;
                break;
            }
            start_tcp_service(bind_addr, sender.clone()).await?;
            start_udp_service(bind_addr, sender.clone()).await?;
            break;
        }
    }

    if opts.is_client() {
        start_tcp_service(bind_addr, sender.clone()).await?;
        start_udp_service(bind_addr, sender).await?;

        // start pac service
        if opts.is_client() {
            if let Some(pac_bind) = opts.client_opts().pac_bind {
                let pac_bind = pac_bind.resolve().await?;
                start_pac_service(pac_bind, opts.bind().as_string()).await?;
            }
        }
    }

    // start monitor service
    if let Some(addr) = opts.monitor() {
        let monitor = Monitor::default();
        set_monitor(monitor);

        let bind_addr = addr.resolve().await?;
        start_monitor_service(bind_addr).await?;
    }

    // load acl
    if let Some(ref path) = opts.acl() {
        let acl = get_acl();

        acl.load_from_file(path).map_err(|err| {
            let msg = format!("[acl] cannot load acl from file due to: {}", err);
            Error::msg(msg)
        })?;

        #[cfg(not(debug_assertions))]
        {
            let path = path.clone();

            tokio::spawn(async move {
                acl.watch(&path).unwrap();
            });
        }
    }

    // consume sockets from receiver
    let handle = tokio::spawn(async move {
        let total_cnt = Arc::new(Counter::default());
        let live_cnt = Arc::new(Counter::default());

        while let Some(socket) = receiver.recv().await {
            if socket.is_none() {
                break;
            }

            let total_cnt = total_cnt.clone();
            let live_cnt = live_cnt.clone();

            total_cnt.inc();
            live_cnt.inc();

            let socket = socket.unwrap();
            let opts = opts.clone();

            // put socket to new task to create a Connection
            tokio::spawn(async move {
                let peer_addr = socket.peer_addr();

                log::info!(
                    "[{}] connected, {} live connections, {} in total",
                    peer_addr,
                    live_cnt,
                    total_cnt
                );

                monitor_log(events::NewConnection {
                    name: "NewConnection",
                    peer_addr,
                    live_cnt: live_cnt.value(),
                    total_cnt: total_cnt.value(),
                });

                let mut conn = Connection::new(socket, opts);

                if let Err(err) = conn.handle().await {
                    log::trace!("{}", err);
                    let _ = conn.close().await;
                }

                live_cnt.dec();

                log::info!(
                    "[{}] closed, {} live connections, {} in total",
                    peer_addr,
                    live_cnt,
                    total_cnt
                );

                monitor_log(events::ConnectionClose {
                    name: "ConnectionClose",
                    peer_addr,
                    live_cnt: live_cnt.value(),
                    total_cnt: total_cnt.value(),
                });
            });
        }
    });

    startup
        .send(Startup::Success(ServiceInfo {
            bind_addr,
            bind_ip,
            bind_host,
            bind_port,
        }))
        .await
        .unwrap();

    handle.await.unwrap();

    Ok(())
}

fn init_tls_configs(opts: &Options) -> Result<()> {
    if opts.is_server() {
        if let (Some(cert), Some(key)) = (opts.tls_cert().as_ref(), opts.server_opts().tls_key.as_ref()) {
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