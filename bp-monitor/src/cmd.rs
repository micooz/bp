use crate::context::Context;
use bytes::Bytes;
use std::convert::TryFrom;

pub enum Command {
    List,
    Unknown(String),
}

impl TryFrom<Bytes> for Command {
    type Error = String;

    fn try_from(buf: Bytes) -> Result<Self, Self::Error> {
        let cmd = String::from_utf8(buf.to_vec()).map_err(|err| format!("parse command failed due to: {}", err))?;

        let cmd = match cmd.trim() {
            "ls" => Command::List,
            unknown => Command::Unknown(unknown.to_string()),
        };

        Ok(cmd)
    }
}

pub fn exec_command(cmd: Command, ctx: Context) {
    match cmd {
        Command::List => {
            todo!()
        }
        Command::Unknown(unknown) => {
            log::error!(
                "[{}] received unknown cmd: {}",
                ctx.peer_addr,
                if unknown.is_empty() { "<empty>".into() } else { unknown }
            );
        }
    }
}
