use crate::{
    cmd::{Command, CommandType},
    context::Context,
};
use bp_lib::net::io::split_tcp_stream;
use std::convert::TryFrom;
use tokio::{net::TcpStream, sync::mpsc::Sender};

pub fn handle_conn(socket: TcpStream, tx: Sender<Command>) {
    tokio::spawn(async move {
        let addr = socket.peer_addr().unwrap();
        let (reader, writer) = split_tcp_stream(socket);

        log::info!("[{}] connected", addr);

        // send a greeting message once client connected
        tx.send(Command {
            peer_addr: addr,
            cmd_type: CommandType::Help,
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

            match CommandType::try_from(buf) {
                Ok(cmd_type) => {
                    tx.send(Command {
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
