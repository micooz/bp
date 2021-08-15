use crate::{
    cmd::{exec_command, Command},
    context::Context,
};
use bp_lib::split_tcp_stream;
use std::convert::TryFrom;
use tokio::net::TcpStream;

pub async fn handle_conn(socket: TcpStream) {
    tokio::spawn(async move {
        let addr = socket.peer_addr().unwrap();

        let (reader, writer) = split_tcp_stream(socket);

        log::info!("[{}] connected", addr);

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
                Ok(cmd) => exec_command(cmd, ctx),
                Err(err) => log::error!("[{}] {}", addr, err),
            }
        }

        log::info!("[{}] disconnected", addr);
    });
}
