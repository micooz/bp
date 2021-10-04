use crate::{
    event::EventSender,
    net::{address::Address, socket},
    protocol::Protocol,
    utils, Result,
};
use async_trait::async_trait;
use bytes::Bytes;

const NOOP: u8 = 0x00;
// const SOCKS_VERSION_V4: u8 = 0x04;
pub const SOCKS_VERSION_V5: u8 = 0x05;
const METHOD_NO_AUTH: u8 = 0x00;
// const METHOD_USERNAME_PASSWORD: u8 = 0x02;
// const METHOD_NOT_ACCEPTABLE: u8 = 0xff;

// const REQUEST_COMMAND_CONNECT: u8 = 0x01;
const REQUEST_COMMAND_BIND: u8 = 0x02;
// const REQUEST_COMMAND_UDP: u8 = 0x03;

pub const ATYP_V4: u8 = 0x01;
pub const ATYP_DOMAIN: u8 = 0x03;
pub const ATYP_V6: u8 = 0x04;

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

pub struct Socks {
    // header_sent: bool,
    bind_addr: Option<Address>,
    proxy_address: Option<Address>,
}

impl Socks {
    pub fn new(bind_addr: Option<Address>) -> Self {
        Self {
            // header_sent: false,
            bind_addr,
            proxy_address: None,
        }
    }
}

impl Clone for Socks {
    fn clone(&self) -> Self {
        Self {
            bind_addr: self.bind_addr.clone(),
            proxy_address: self.proxy_address.clone(),
        }
    }
}

#[async_trait]
impl Protocol for Socks {
    fn get_name(&self) -> String {
        "socks".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<(Address, Option<Bytes>)> {
        if socket.is_udp() {
            // Socks5 UDP Request/Response
            // +----+------+------+----------+----------+----------+
            // |RSV | FRAG | ATYP | DST.ADDR | DST.PORT |   DATA   |
            // +----+------+------+----------+----------+----------+
            // | 2  |  1   |  1   | Variable |    2     | Variable |
            // +----+------+------+----------+----------+----------+
            let packet = socket.read_buf(1500).await?;
            let buf = packet.slice(3..);

            return Address::from_bytes(buf);
        }

        let reader = socket.tcp_reader();
        let mut reader = reader.lock().await;

        let writer = socket.tcp_writer();
        let mut writer = writer.lock().await;

        // 1. Parse Socks5 Identifier Message

        // Socks5 Identifier Message
        // +----+----------+----------+
        // |VER | NMETHODS | METHODS  |
        // +----+----------+----------+
        // | 1  |    1     | 1 to 255 |
        // +----+----------+----------+

        // check the first two bytes
        let buf = reader.read_exact(2).await?;
        let n_methods = buf[1] as usize;

        if buf[0] != SOCKS_VERSION_V5 || n_methods < 1 {
            return Err(format!(
                "message is invalid when parsing socks5 identifier message: {}",
                utils::fmt::ToHex(buf.to_vec())
            )
            .into());
        }

        // select one method
        let buf = reader.read_exact(n_methods).await?;

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
            return Err(format!(
                "METHOD only support {:#04x} but it's not found in socks5 identifier message",
                METHOD_NO_AUTH
            )
            .into());
        }

        // 2. Reply Socks5 Select Message

        // Socks5 Select Message
        // +----+--------+
        // |VER | METHOD |
        // +----+--------+
        // | 1  |   1    |
        // +----+--------+

        writer.write(&[SOCKS_VERSION_V5, METHOD_NO_AUTH]).await?;

        // 3. Parse Socks5 Request Message

        // Socks5 Request Message
        // +----+-----+-------+------+----------+----------+
        // |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
        // +----+-----+-------+------+----------+----------+
        // | 1  |  1  | X'00' |  1   | Variable |    2     |
        // +----+-----+-------+------+----------+----------+

        let buf = reader.read_exact(3).await?;

        if buf[0] != SOCKS_VERSION_V5 {
            return Err(format!("VER should be {:#04x} but got {:#04x}", SOCKS_VERSION_V5, buf[0]).into());
        }

        // TODO: add support for REQUEST_COMMAND_BIND
        if buf[1] == REQUEST_COMMAND_BIND {
            return Err(format!("CMD does not support {:#04x}", REQUEST_COMMAND_BIND).into());
        }

        if buf[2] != NOOP {
            return Err(format!("RSV must be 0x00 but got {:#04x}", buf[2]).into());
        }

        let addr = Address::from_reader(&mut reader).await?;

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

        writer.write(reply_buf.as_slice()).await?;

        Ok((addr, None))
    }

    async fn client_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
        // let mut frame = BytesMut::new();

        // if !self.header_sent {
        //     let header = self.proxy_address.as_ref().unwrap();
        //     frame.put(header.as_bytes());
        //     self.header_sent = true;
        // }

        // let buf = reader.read_buf(reader, 1024).await?;
        // frame.put(buf);

        // tx.send(Event::EncodeDone(frame.freeze())).await?;

        // Ok(())
    }

    async fn server_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }
}
