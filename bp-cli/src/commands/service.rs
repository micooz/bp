use std::{future::Future, sync::Arc, time::Duration};

use anyhow::{Error, Result};
use bp_core::{
    acl::get_acl, init_dns_resolver, init_quic_endpoint_pool, init_quinn_client_config, init_quinn_server_config,
    init_tls_client_config, init_tls_server_config, monitor_log, set_monitor, start_monitor_service, start_pac_service,
    start_quic_service, start_tcp_service, start_tls_service, start_udp_service, Connection, Options, ServiceInfo,
    ServiceProtocol, Shutdown, Socket, Startup,
};
use bp_monitor::{events, Monitor};
use tokio::sync::mpsc;

use crate::{dirs::Dirs, utils::counter::Counter};

type StartupSender = mpsc::Sender<Startup>;

pub async fn run(mut opts: Options, startup: StartupSender, shutdown: impl Future) {
    let fail = |err: Error| async {
        log::error!("{}", err);
        startup.send(Startup::Fail(err)).await.unwrap();
    };

    // try load bp service options from --config
    if let Some(config) = opts.config() {
        log::info!("loading configuration from {}", config);

        if let Err(err) = opts.try_load_from_file(&config) {
            return fail(err).await;
        }
    }

    // check options
    if let Err(err) = opts.check() {
        return fail(err).await;
    }

    let inner_shutdown = Shutdown::new();

    // bootstrap bp service
    tokio::select! {
        Err(err) = boot(opts, startup.clone(), inner_shutdown.clone()) => {
            fail(err).await;
        }
        _ = shutdown => {
            log::info!("gracefully shutting down...");
            let count = inner_shutdown.broadcast();
            log::info!("informed {} receivers, waiting for 2 seconds...", count);
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    };
}

const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

async fn boot(opts: Options, startup: StartupSender, shutdown: Shutdown) -> Result<()> {
    // pre boot
    pre_boot(&opts).await?;
    // start services
    start_services(opts.clone(), startup, shutdown).await?;

    Ok(())
}

async fn pre_boot(opts: &Options) -> Result<()> {
    // dirs init
    Dirs::init()?;

    log::info!("log files are stored at logs/bp.log");

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

async fn start_services(opts: Options, startup: StartupSender, shutdown: Shutdown) -> Result<()> {
    let (sender, mut receiver) = mpsc::channel::<Option<Socket>>(SERVICE_CONNECTION_THRESHOLD);

    let mut services = vec![];

    let bind_addr = opts.bind().resolve().await?;
    let bind_ip = bind_addr.ip().to_string();
    let bind_host = opts.bind().host();
    let bind_port = opts.bind().port();

    macro_rules! add_service {
        ($protocol:expr) => {{
            services.push(ServiceInfo {
                protocol: $protocol,
                bind_addr: bind_addr.clone(),
                bind_host: bind_host.clone(),
                bind_ip: bind_ip.clone(),
                bind_port,
            });
        }};
    }

    if opts.is_server() {
        // server side enable --tls, start TLS service
        if opts.tls() {
            start_tls_service(bind_addr, sender.clone(), shutdown.clone()).await?;
            start_udp_service(bind_addr, sender.clone(), shutdown.clone()).await?;
            add_service!(ServiceProtocol::Tls);
            add_service!(ServiceProtocol::Udp);
        }
        // server side enable --quic, start QUIC service
        else if opts.quic() {
            start_quic_service(bind_addr, sender.clone(), shutdown.clone()).await?;
            add_service!(ServiceProtocol::Quic);
        }
        // others
        else {
            start_tcp_service(bind_addr, sender.clone(), shutdown.clone()).await?;
            start_udp_service(bind_addr, sender.clone(), shutdown.clone()).await?;
            add_service!(ServiceProtocol::Tcp);
            add_service!(ServiceProtocol::Udp);
        }
    }

    if opts.is_client() {
        start_tcp_service(bind_addr, sender.clone(), shutdown.clone()).await?;
        start_udp_service(bind_addr, sender, shutdown.clone()).await?;

        add_service!(ServiceProtocol::Tcp);
        add_service!(ServiceProtocol::Udp);

        let opts = opts.client_opts();

        // start pac service
        if let Some(addr) = opts.pac_bind.clone() {
            let bind_addr = addr.resolve().await?;

            // fallback pac proxy target to --bind
            let pac_proxy = opts.pac_proxy.unwrap_or(opts.bind);

            start_pac_service(bind_addr, pac_proxy.to_string(), shutdown.clone()).await?;

            services.push(ServiceInfo {
                protocol: ServiceProtocol::Pac,
                bind_addr,
                bind_host: addr.host(),
                bind_ip: bind_addr.ip().to_string(),
                bind_port: bind_addr.port(),
            });
        }
    }

    // start monitor service
    if let Some(addr) = opts.monitor() {
        let monitor = Monitor::default();
        set_monitor(monitor);

        let bind_addr = addr.resolve().await?;
        start_monitor_service(bind_addr, shutdown.clone()).await?;

        services.push(ServiceInfo {
            protocol: ServiceProtocol::Monitor,
            bind_addr,
            bind_host: addr.host(),
            bind_ip: bind_addr.ip().to_string(),
            bind_port: bind_addr.port(),
        });
    }

    // load acl
    if let Some(ref path) = opts.acl() {
        let acl = get_acl();

        acl.load_from_file(path).map_err(|err| {
            let msg = format!("[acl] cannot load acl from file due to: {}", err);
            Error::msg(msg)
        })?;

        #[cfg(not(test))]
        {
            let path = path.clone();
            let shutdown = shutdown.clone();

            tokio::spawn(async move {
                acl.watch(&path, shutdown).unwrap();
            });
        }
    }

    // consume sockets from receiver
    let handle = tokio::spawn(async move {
        let total_cnt = Arc::new(Counter::default());
        let live_cnt = Arc::new(Counter::default());

        while let Some(socket) = tokio::select! {
            v = receiver.recv() => v,
            _ = shutdown.recv() => None,
        } {
            if socket.is_none() {
                break;
            }

            let total_cnt = total_cnt.clone();
            let live_cnt = live_cnt.clone();

            total_cnt.inc();
            live_cnt.inc();

            let socket = socket.unwrap();
            let opts = opts.clone();
            let shutdown = shutdown.clone();

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

                let mut conn = Connection::new(socket, opts, shutdown);

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

    startup.send(Startup::Success(services)).await.unwrap();

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
