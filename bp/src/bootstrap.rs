use crate::options::Options;
use bp_lib::{start_service, Connection, ConnectionOptions};
use bp_monitor::ConnectionRecord;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

#[cfg(feature = "monitor")]
use bp_monitor::{handle_conn as handle_monitor_conn, MonitorCommand, SharedContext};

pub async fn bootstrap(opts: Options) -> std::io::Result<()> {
    let bind_addr = opts.bind.clone();

    let shared_conns = Arc::new(RwLock::new(HashSet::<ConnectionRecord>::new()));

    #[cfg(feature = "monitor")]
    start_monitor_service(
        opts.clone(),
        // put some shared data to context,
        // so that monitor can process these data.
        SharedContext {
            shared_conns: shared_conns.clone(),
        },
    );

    // start local service
    let task_main = tokio::spawn(async move {
        let mut handler = start_service(bind_addr, "main");
        let mut id = 0usize;

        while let Some(socket) = handler.recv().await {
            id = id + 1;
            handle_main_conn(id, socket, opts.clone(), shared_conns.clone());
        }
    });

    task_main.await?;

    Ok(())
}

fn handle_main_conn(id: usize, socket: TcpStream, opts: Options, shared_conns: Arc<RwLock<HashSet<ConnectionRecord>>>) {
    let addr = socket.peer_addr().unwrap();
    let conn_opts = ConnectionOptions::new(
        opts.get_service_type().unwrap(),
        opts.protocol,
        opts.key,
        opts.server_host,
        opts.server_port,
    );

    let conn = Arc::new(RwLock::new(Connection::new(socket, conn_opts)));

    tokio::spawn(async move {
        log::info!("[{}] connected", addr);

        // store conn to shared_conns
        let record = ConnectionRecord::new(id, conn.clone());

        {
            RwLock::write(&shared_conns).await.insert(record.clone());
        }

        let conn = conn.clone();
        let mut conn = RwLock::write(&conn).await;

        if let Err(err) = conn.handle().await {
            log::error!("{}", err);
            let _ = conn.force_close().await;
        }

        log::info!("[{}] disconnected", addr);

        // TODO: remove conn from shared_conns when connection closed
        // #[cfg(feature = "monitor")]
        // RwLock::write(&shared_conns).await.remove(&record);
    });
}

#[cfg(feature = "monitor")]
fn start_monitor_service(opts: Options, ctx: SharedContext) {
    use tokio::sync::mpsc;

    let bind_addr_monitor = opts.get_monitor_bind_addr();

    let (tx_cmd, mut rx_cmd) = mpsc::channel::<MonitorCommand>(32);

    // start monitor service
    tokio::spawn(async move {
        let mut handler = start_service(bind_addr_monitor, "monitor");

        while let Some(socket) = handler.recv().await {
            handle_monitor_conn(socket, tx_cmd.clone()).await
        }
    });

    // recv client command
    tokio::spawn(async move {
        while let Some(mut mc) = rx_cmd.recv().await {
            mc.exec(ctx.clone()).await;
        }
    });
}
