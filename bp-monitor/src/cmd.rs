use crate::{context::Context, SharedContext};
use bytes::Bytes;
use std::{convert::TryFrom, fmt::Display, net::SocketAddr};
use tokio::sync::RwLock;

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

#[derive(Debug)]
pub enum Command {
    Help,
    List,
    Unknown(String),
}

impl From<String> for Command {
    fn from(s: String) -> Self {
        match s.trim() {
            "help" | "h" => Command::Help,
            "ls" => Command::List,
            unknown => Command::Unknown(unknown.to_string()),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Command::Help => "help",
            Command::List => "ls",
            Command::Unknown(cmd) => cmd.as_str(),
        };
        f.write_str(s)?;
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
        log::info!("[{}] execute command: <{}>", self.peer_addr, self.cmd);
        match &self.cmd {
            Command::Help => {
                self.reply(format!("\n{}", include_str!("help.txt"))).await;
            }
            Command::List => {
                let mut list = vec![];

                for conn in RwLock::read(&ctx.shared_conns).await.iter() {
                    let snapshot = RwLock::read(conn).await.snapshot();
                    list.push(snapshot);
                }

                if list.is_empty() {
                    self.reply(String::from("<no alive connections>")).await;
                    return;
                }

                let msg: String = list
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| format!("[{}] {}\n", i, s.get_abstract()))
                    .collect();

                self.reply(msg).await;
            }
            Command::Unknown(cmd) => {
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
