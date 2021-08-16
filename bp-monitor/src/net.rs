use crate::{
    cmd::{Command, MonitorCommand},
    context::Context,
};
use bp_lib::split_tcp_stream;
use std::convert::TryFrom;
use tokio::{net::TcpStream, sync::mpsc::Sender};

pub async fn handle_conn(socket: TcpStream, tx: Sender<MonitorCommand>) {
    tokio::spawn(async move {
        let addr = socket.peer_addr().unwrap();
        let (reader, writer) = split_tcp_stream(socket);

        log::info!("[{}] connected", addr);

        // send a greeting message once client connected
        tx.send(MonitorCommand {
            peer_addr: addr,
            cmd: Command::Help,
            ctx: Context {
                peer_addr: addr,
                writer: writer.clone(),
            },
        })
        .await
        .unwrap();

        loop {
            let mut reader = reader.lock().await;
            let res = reader.read_buf(32).await;

            if let Err(err) = res {
                log::error!("{}", err);
                break;
            }

            let ctx = Context {
                peer_addr: addr,
                writer: writer.clone(),
            };

            let buf = res.unwrap();

            match Command::try_from(buf) {
                Ok(cmd) => {
                    tx.send(MonitorCommand {
                        peer_addr: addr,
                        cmd,
                        ctx,
                    })
                    .await
                    .unwrap();
                }
                Err(err) => log::error!("[{}] {}", addr, err),
            }
        }

        log::info!("[{}] disconnected", addr);
    });
}
