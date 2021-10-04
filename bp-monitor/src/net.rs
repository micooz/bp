use crate::{
    cmd::{CommandType, MonitorCommand},
    context::Context,
};
use bp_lib::net::socket;
use std::convert::TryFrom;
use tokio::sync::mpsc::Sender;

pub fn handle_conn(socket: socket::Socket, tx: Sender<MonitorCommand>) {
    tokio::spawn(async move {
        let addr = socket.peer_addr().unwrap();

        log::info!("[{}] connected", addr);

        // send a greeting message once client connected
        tx.send(MonitorCommand {
            peer_addr: addr,
            cmd_type: CommandType::Help,
            ctx: Context {
                peer_addr: addr,
                writer: socket.tcp_writer(),
            },
        })
        .await
        .unwrap();

        loop {
            let reader = socket.tcp_reader();
            let mut reader = reader.lock().await;
            let res = reader.read_buf(32).await;

            if let Err(err) = res {
                log::error!("{}", err);
                break;
            }

            let ctx = Context {
                peer_addr: addr,
                writer: socket.tcp_writer(),
            };

            let buf = res.unwrap();

            match CommandType::try_from(buf) {
                Ok(cmd_type) => {
                    tx.send(MonitorCommand {
                        peer_addr: addr,
                        cmd_type,
                        ctx,
                    })
                    .await
                    .unwrap();
                }
                Err(err) => {
                    if !err.is_empty() {
                        log::error!("[{}] {}", addr, err);
                    }
                }
            }
        }

        log::info!("[{}] disconnected", addr);
    });
}
