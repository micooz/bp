use crate::options::Options;
use bp_lib::{start_service, Connection, ConnectionOptions};
use tokio::net::TcpStream;

#[cfg(feature = "monitor")]
use bp_monitor::handle_conn as handle_monitor_conn;

pub async fn bootstrap(opts: Options) -> std::io::Result<()> {
    let bind_addr = opts.bind.clone();

    #[cfg(feature = "monitor")]
    let bind_addr_monitor = opts.get_monitor_bind_addr();

    // start local service
    let task_main = tokio::spawn(async move {
        let mut handler = start_service(bind_addr, "main");

        while let Some(socket) = handler.recv().await {
            handle_main_conn(socket, opts.clone()).await;
        }
    });

    // start monitor service
    #[cfg(feature = "monitor")]
    let _task_monitor = tokio::spawn(async move {
        let mut handler = start_service(bind_addr_monitor, "monitor");

        while let Some(socket) = handler.recv().await {
            handle_monitor_conn(socket).await
        }
    });

    task_main.await?;
    // task_monitor.await?;

    Ok(())
}

async fn handle_main_conn(socket: TcpStream, opts: Options) {
    let addr = socket.peer_addr().unwrap();
    let conn_opts = ConnectionOptions::new(
        opts.get_service_type().unwrap(),
        opts.protocol,
        opts.key,
        opts.server_host,
        opts.server_port,
    );

    let mut conn = Connection::new(socket, conn_opts);

    tokio::spawn(async move {
        log::info!("[{}] connected", addr);

        if let Err(err) = conn.handle().await {
            log::error!("{}", err);
            let _ = conn.force_close().await;
        }

        log::info!("[{}] disconnected", addr);
    });
}
