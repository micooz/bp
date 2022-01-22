use anyhow::{Error, Result};
use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    net::{address::Address, socket::Socket},
    proto::{Protocol, ProtocolType, ResolvedResult},
    utils,
};

const NOOP: u8 = 0x00;
// const SOCKS_VERSION_V4: u8 = 0x04;
pub const SOCKS_VERSION_V5: u8 = 0x05;
const METHOD_NO_AUTH: u8 = 0x00;
// const METHOD_USERNAME_PASSWORD: u8 = 0x02;
// const METHOD_NOT_ACCEPTABLE: u8 = 0xff;

// const REQUEST_COMMAND_CONNECT: u8 = 0x01;
const REQUEST_COMMAND_BIND: u8 = 0x02;
// const REQUEST_COMMAND_UDP: u8 = 0x03;

const ATYP_V4: u8 = 0x01;
// const ATYP_DOMAIN: u8 = 0x03;
// const ATYP_V6: u8 = 0x04;

// const REPLY_GRANTED: u8 = 0x5a;
const REPLY_SUCCEEDED: u8 = 0x00;
// const REPLY_FAILURE: u8 = 0x01;
// const REPLY_NOT_ALLOWED: u8 = 0x02;
// const REPLY_NETWORK_UNREACHABLE: u8 = 0x03;
// const REPLY_HOST_UNREACHABLE: u8 = 0x04;
// const REPLY_CONNECTION_REFUSED: u8 = 0x05;
// const REPLY_TTL_EXPIRED: u8 = 0x06;
// const REPLY_COMMAND_NOT_SUPPORTED: u8 = 0x07;
// const REPLY_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
// const REPLY_UNASSIGNED: u8 = 0xff;

#[derive(Clone)]
pub struct Socks {
    bind_addr: Option<Address>,
    resolved_result: Option<ResolvedResult>,
}

impl Socks {
    pub fn new(bind_addr: Option<Address>) -> Self {
        Self {
            bind_addr,
            resolved_result: None,
        }
    }
}

#[async_trait]
impl Protocol for Socks {
    fn get_name(&self) -> String {
        "socks".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> Option<&ResolvedResult> {
        self.resolved_result.as_ref()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<()> {
        if socket.is_udp() {
            // Socks5 UDP Request/Response
            // +----+------+------+----------+----------+----------+
            // |RSV | FRAG | ATYP | DST.ADDR | DST.PORT |   DATA   |
            // +----+------+------+----------+----------+----------+
            // | 2  |  1   |  1   | Variable |    2     | Variable |
            // +----+------+------+----------+----------+----------+
            let packet = socket.read_some().await?;
            let buf = packet.slice(3..);

            let (address, pending_buf) = Address::from_bytes(buf)?;

            self.set_resolved_result(ResolvedResult {
                protocol: ProtocolType::Socks,
                address,
                pending_buf,
            });

            return Ok(());
        }

        // 1. Parse Socks5 Identifier Message

        // Socks5 Identifier Message
        // +----+----------+----------+
        // |VER | NMETHODS | METHODS  |
        // +----+----------+----------+
        // | 1  |    1     | 1 to 255 |
        // +----+----------+----------+

        // check the first two bytes
        let buf = socket.read_exact(2).await?;
        let n_methods = buf[1] as usize;

        if buf[0] != SOCKS_VERSION_V5 || n_methods < 1 {
            return Err(Error::msg(format!(
                "message is invalid when parsing socks5 identifier message: {}",
                utils::fmt::ToHex(buf.to_vec())
            )));
        }

        // select one method
        let buf = socket.read_exact(n_methods).await?;

        let mut method = None;
        let mut n = 0usize;

        // TODO: now only support METHOD_NO_AUTH
        while n < n_methods as usize {
            if buf[n] == METHOD_NO_AUTH {
                method = Some(METHOD_NO_AUTH);
                break;
            }
            n += 1;
        }

        if method.is_none() {
            return Err(Error::msg(format!(
                "METHOD only support {:#04x} but it's not found in socks5 identifier message",
                METHOD_NO_AUTH
            )));
        }

        // 2. Reply Socks5 Select Message

        // Socks5 Select Message
        // +----+--------+
        // |VER | METHOD |
        // +----+--------+
        // | 1  |   1    |
        // +----+--------+

        socket.send(&[SOCKS_VERSION_V5, METHOD_NO_AUTH]).await?;

        // 3. Parse Socks5 Request Message

        // Socks5 Request Message
        // +----+-----+-------+------+----------+----------+
        // |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
        // +----+-----+-------+------+----------+----------+
        // | 1  |  1  | X'00' |  1   | Variable |    2     |
        // +----+-----+-------+------+----------+----------+

        let buf = socket.read_exact(3).await?;

        if buf[0] != SOCKS_VERSION_V5 {
            return Err(Error::msg(format!(
                "VER should be {:#04x} but got {:#04x}",
                SOCKS_VERSION_V5, buf[0]
            )));
        }

        // TODO: add support for REQUEST_COMMAND_BIND
        if buf[1] == REQUEST_COMMAND_BIND {
            return Err(Error::msg(format!(
                "CMD does not support {:#04x}",
                REQUEST_COMMAND_BIND
            )));
        }

        if buf[2] != NOOP {
            return Err(Error::msg(format!("RSV must be 0x00 but got {:#04x}", buf[2])));
        }

        let addr = Address::from_socket(socket).await?;

        // 4. Reply Socks5 Reply Message

        // Socks5 Reply Message
        // +----+-----+-------+------+----------+----------+
        // |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
        // +----+-----+-------+------+----------+----------+
        // | 1  |  1  | X'00' |  1   | Variable |    2     |
        // +----+-----+-------+------+----------+----------+

        let mut reply_buf = vec![SOCKS_VERSION_V5, REPLY_SUCCEEDED, NOOP];

        match &self.bind_addr {
            Some(addr) => {
                let mut addr_buf = addr.as_bytes().to_vec();
                reply_buf.append(&mut addr_buf);
            }
            None => {
                let mut addr_buf = [ATYP_V4, NOOP, NOOP, NOOP, NOOP, NOOP, NOOP].to_vec();
                reply_buf.append(&mut addr_buf);
            }
        }

        socket.send(reply_buf.as_slice()).await?;

        self.set_resolved_result(ResolvedResult {
            protocol: ProtocolType::Socks,
            address: addr,
            pending_buf: None,
        });

        Ok(())
    }

    async fn client_encode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &Socket) -> Result<Bytes> {
        unimplemented!()
    }
}
