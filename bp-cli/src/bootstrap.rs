use std::{env, process::Command};

use anyhow::{Error, Result};
use bp_core::{global, init_dns_resolver, start_service, Connection, Options, StartupInfo};
use tokio::{sync::oneshot::Sender, task::JoinHandle, time};

use crate::dirs::Dirs;

const ENV_DISABLE_DAEMONIZE: &str = "DISABLE_DAEMONIZE";

pub async fn bootstrap(opts: Options, sender_ready: Sender<StartupInfo>) -> Result<()> {
    // dirs init
    Dirs::init()?;

    log::info!("log files are stored at {}", Dirs::log_file().to_str().unwrap());

    // daemonize
    #[cfg(target_family = "unix")]
    if opts.daemonize && !env::vars().any(|(k, _)| k == ENV_DISABLE_DAEMONIZE) {
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

        return Ok(());
    }

    // dns server
    init_dns_resolver(opts.get_dns_server().as_socket_addr()).await?;

    // monitor service
    #[cfg(feature = "monitor")]
    {
        use bp_monitor::MonitorCommand;

        let (tx, rx) = tokio::sync::mpsc::channel::<MonitorCommand>(32);

        #[cfg(feature = "monitor")]
        start_monitor_service(opts.clone(), tx);

        tokio::spawn(async move {
            while let Some(mut cmd) = rx.recv().await {
                cmd.exec(shared_data_monitor.clone()).await;
            }
        });
    }

    // main service
    let handle = start_main_service(opts.clone()).await?;

    sender_ready
        .send(StartupInfo {
            bind_addr: opts.bind.clone(),
        })
        .unwrap();

    handle.await.unwrap();

    Ok(())
}

async fn start_main_service(opts: Options) -> Result<JoinHandle<()>> {
    let mut receiver = start_service("main", &opts.bind).await?;

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
            if socket.is_none() {
                break;
            }

            id += 1;

            let socket = socket.unwrap();
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

#[cfg(target_family = "unix")]
fn daemonize() -> Result<()> {
    use std::fs::File;

    use daemonize::Daemonize;

    let stdout = File::create(Dirs::run_daemon_out()).unwrap();
    let stderr = File::create(Dirs::run_daemon_err()).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(Dirs::run_pid()) // Every method except `new` and `start`
        // .chown_pid_file(true) // is optional, see `Daemonize` documentation
        // .working_directory("/tmp") // for default behaviour.
        // .user("root")
        // .group("root") // Group name
        // .group(2) // or group id.
        // .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    // .exit_action(|| println!("Executed before master process exits"))
    // .privileged_action(|| "Executed before drop privileges");

    daemonize.start().map(|_| ()).map_err(|err| Error::msg(err.to_string()))
}

#[cfg(feature = "monitor")]
fn start_monitor_service(opts: Options, tx: sync::mpsc::Sender<bp_monitor::MonitorCommand>) {
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
