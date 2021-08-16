use crate::{context::Context, SharedContext};
use bytes::Bytes;
use std::{convert::TryFrom, fmt::Display, net::SocketAddr};
use tokio::sync::RwLock;

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

#[derive(Debug)]
pub enum Command {
    Help,
    List,
    Dump(usize, u16),
    Invalid(String),
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        let mut parts = s.trim().split_whitespace();
        let cmd_str = parts.next().unwrap_or("");

        match cmd_str {
            "help" | "h" => Command::Help,
            "ls" => Command::List,
            "dump" => {
                // parse n
                let n = match parts.next().unwrap_or("").parse::<usize>() {
                    Ok(n) => n,
                    Err(_) => {
                        return Command::Invalid(s);
                    }
                };
                // parse k
                let k: u16 = parts.next().unwrap_or("").parse().unwrap_or(0);
                Command::Dump(n, k)
            }
            invalid => Command::Invalid(invalid.to_string()),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Command::Help => "help".to_string(),
            Command::List => "ls".to_string(),
            Command::Dump(n, k) => format!("dump {} {}", n, k),
            Command::Invalid(cmd) => cmd.clone(),
        };
        f.write_str(s.as_str())?;
        Ok(())
    }
}

impl TryFrom<Bytes> for Command {
    type Error = String;

    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let cmd = String::from_utf8(buf.to_vec()).map_err(|err| format!("parse command failed due to: {}", err))?;
        Ok(cmd.into())
    }
}

#[derive(Debug)]
pub struct MonitorCommand {
    pub peer_addr: SocketAddr,
    pub cmd: Command,
    pub ctx: Context,
}

impl MonitorCommand {
    pub async fn reply(&mut self, data: String) {
        let mut writer = self.ctx.writer.lock().await;

        writer.write(data.as_bytes()).await.unwrap();
        writer.write(LINE_ENDING.as_bytes()).await.unwrap();
    }

    pub async fn exec(&mut self, ctx: SharedContext) {
        match self.cmd {
            Command::Invalid(_) => {}
            _ => {
                log::info!("[{}] execute command: <{}>", self.peer_addr, self.cmd);
            }
        }

        match &self.cmd {
            Command::Help => {
                self.reply(format!("\n{}", include_str!("help.txt"))).await;
            }
            Command::List => {
                let shared_conns = RwLock::read(&ctx.shared_conns).await;
                let mut snapshot_list = vec![];

                if shared_conns.is_empty() {
                    self.reply(String::from("<no alive connections>")).await;
                    return;
                }

                for (&index, conn) in shared_conns.iter() {
                    let conn = RwLock::read(conn).await;
                    let snapshot = conn.snapshot();
                    snapshot_list.push((index, snapshot));
                }

                let msg: String = snapshot_list
                    .into_iter()
                    .map(|(i, v)| format!("[{}] {}\n", i, v.get_abstract()))
                    .collect();

                self.reply(msg).await;
            }
            Command::Dump(n, k) => {
                let shared_conns = RwLock::read(&ctx.shared_conns).await;

                match shared_conns.get(&n) {
                    Some(conn) => {
                        // TODO: make better hexdump
                        let conn = RwLock::read(conn).await;
                        let mut buf = conn.dump();
                        if *k > 0 {
                            let len = std::cmp::min(*k as usize, buf.len());
                            buf = buf.slice(0..len);
                        }
                        let buf = buf.to_vec();

                        self.reply(String::from_utf8_lossy(&buf[..]).into()).await;
                    }
                    None => {
                        self.reply(format!("connection with index {} is not found", n)).await;
                    }
                }
            }
            Command::Invalid(cmd) => {
                // ignore \n, \r\n chars
                if cmd.is_empty() {
                    return;
                }
                let msg = format!("unsupported command: {}", cmd);
                self.reply(msg).await;
            }
        }
    }
}
