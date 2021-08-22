use crate::context::Context;
// use bp_lib::net::{MonitorCommand, MonitorCommandParam, MonitorCommandReturn};
use bytes::Bytes;
use std::{convert::TryFrom, fmt::Display, net::SocketAddr};

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

#[derive(Debug)]
pub enum CommandType {
    Help,
    List,
    Dump(usize, u16),
}

impl Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Help => "help".to_string(),
            Self::List => "ls".to_string(),
            Self::Dump(n, k) => format!("dump {} {}", n, k),
        };
        f.write_str(s.as_str())?;
        Ok(())
    }
}

impl TryFrom<Bytes> for CommandType {
    type Error = String;

    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let cmd = String::from_utf8(buf.to_vec()).map_err(|err| format!("parse command failed due to: {}", err))?;

        let mut parts = cmd.trim().split_whitespace();
        let cmd_str = parts.next().unwrap_or("");

        let value = match cmd_str {
            "help" | "h" => Self::Help,
            "ls" => Self::List,
            "dump" => {
                // parse n
                let n = match parts.next().unwrap_or("").parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => return Err("n must be specified".into()),
                };
                // parse k
                let k: u16 = parts.next().unwrap_or("").parse().unwrap_or(0);
                Self::Dump(n, k)
            }
            other => {
                // ignore \n, \r\n chars
                if other.is_empty() {
                    return Err("".into());
                }
                return Err(format!("unsupported command: {}", other));
            }
        };

        Ok(value)
    }
}

#[derive(Debug)]
pub struct Command {
    pub peer_addr: SocketAddr,
    pub cmd_type: CommandType,
    pub ctx: Context,
}

impl Command {
    pub async fn reply(&mut self, data: String) {
        let mut writer = self.ctx.writer.lock().await;
        writer.write(data.as_bytes()).await.unwrap();
        writer.write(LINE_ENDING.as_bytes()).await.unwrap();
    }

    pub async fn exec(&mut self) {
        log::info!("[{}] execute command: <{}>", self.peer_addr, self.cmd_type);

        match &self.cmd_type {
            CommandType::Help => {
                self.reply(format!("\n{}", include_str!("help.txt"))).await;
            }
            CommandType::List => {
                // let res = emitter.emit("snapshot", None).await.unwrap();

                // if let MonitorCommandReturn::Snapshot(snapshot) = res {
                // snapshot.
                // }

                // let (txx, mut rxx) = mpsc::channel(1);

                // if tx.send(MonitorCommandForConnection::Snapshot(txx)).is_err() {
                //     return;
                // }

                // let mut snapshot_list = vec![];

                // while let Some(x) = rxx.recv().await {
                //     snapshot_list.push(x);
                // }

                // let msg: String = snapshot_list
                //     .into_iter()
                //     .map(|v| format!("[{}] {}\n", v.id(), v.get_abstract()))
                //     .collect();

                // self.reply(msg).await;
            }
            CommandType::Dump(_n, _k) => {
                // let buf = emitter
                //     .emit("dump", Some(MonitorCommandParam::Dump(*n, *k)))
                //     .await
                //     .unwrap();

                // if let MonitorCommandReturn::Dump(buf) = buf {
                //     self.reply(String::from_utf8_lossy(&buf.unwrap()[..]).into()).await;
                // }

                // self.reply(format!("connection with index {} is not found", n)).await;
            }
        }
    }
}
