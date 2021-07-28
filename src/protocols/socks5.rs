use super::{
    super::net::address::{Host, NetAddr},
    super::utils::ToHex,
    Protocol, Result, TcpStreamReader, TcpStreamWriter,
};
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use std::{
    cell::Cell,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Mutex,
    vec,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const NOOP: u8 = 0x00;
// const SOCKS_VERSION_V4: u8 = 0x04;
const SOCKS_VERSION_V5: u8 = 0x05;
const METHOD_NO_AUTH: u8 = 0x00;
// const METHOD_USERNAME_PASSWORD: u8 = 0x02;
// const METHOD_NOT_ACCEPTABLE: u8 = 0xff;

const REQUEST_COMMAND_CONNECT: u8 = 0x01;
// const REQUEST_COMMAND_BIND: u8 = 0x02;
// const REQUEST_COMMAND_UDP: u8 = 0x03;

const ATYP_V4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_V6: u8 = 0x04;

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

pub struct Socks5 {
    header_sent: Mutex<Cell<bool>>,

    proxy_address: Option<NetAddr>,
}

impl Socks5 {
    pub fn new() -> Self {
        Socks5 {
            header_sent: Mutex::new(Cell::new(false)),
            proxy_address: None,
        }
    }
}

#[async_trait]
impl Protocol for Socks5 {
    fn get_name(&self) -> String {
        "socks5".into()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr> {
        // 1. Parse Socks5 Identifier Message

        // Socks5 Identifier Message
        // +----+----------+----------+
        // |VER | NMETHODS | METHODS  |
        // +----+----------+----------+
        // | 1  |    1     | 1 to 255 |
        // +----+----------+----------+

        // check the first two bytes
        let mut buf = vec![0u8; 2];
        reader.read_exact(&mut buf).await?;

        let n_methods = buf[1] as usize;

        if buf[0] != SOCKS_VERSION_V5 || n_methods < 1 {
            return Err(format!(
                "message is invalid when parsing socks5 identifier message: {}",
                ToHex(buf)
            )
            .into());
        }

        // select one method
        let mut buf = vec![0u8; n_methods];
        reader.read_exact(&mut buf).await?;

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

        let mut buf = vec![0u8; 4];
        reader.read_exact(&mut buf).await?;

        if buf[0] != SOCKS_VERSION_V5 {
            return Err(format!(
                "VER should be {:#04x} but got {:#04x}",
                SOCKS_VERSION_V5, buf[0]
            )
            .into());
        }

        // TODO: only support REQUEST_COMMAND_CONNECT
        if buf[1] != REQUEST_COMMAND_CONNECT {
            return Err(format!(
                "CMD only support {:#04x} but got {:#04x}",
                REQUEST_COMMAND_CONNECT, buf[1]
            )
            .into());
        }

        if buf[2] != NOOP {
            return Err(format!("RSV must be 0x00 but got {:#04x}", buf[2]).into());
        }

        let addr = match buf[3] {
            ATYP_V4 => {
                // read ipv4 address and port
                buf.resize(4 + 2, 0);
                reader.read_exact(&mut buf).await?;

                let ip = IpAddr::V4(Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]));
                let port: u16 = u16::from_be_bytes([buf[4], buf[5]]);

                Some(NetAddr::new(Host::Ip(ip), port))
            }
            ATYP_DOMAIN => {
                // read hostname length
                let mut buf = vec![0u8; 1];
                reader.read_exact(&mut buf).await?;
                let len = buf[0];

                // read hostname
                buf.resize(len as usize, 0);
                reader.read_exact(&mut buf).await?;

                let hostname = String::from_utf8(buf).unwrap();

                // read port
                let mut buf = vec![0u8; 2];
                reader.read_exact(&mut buf).await?;
                let port: u16 = u16::from_be_bytes([buf[0], buf[1]]);

                Some(NetAddr::new(Host::Name(hostname), port))
            }
            ATYP_V6 => {
                // read ipv6 and port
                buf.resize(16 + 2, 0);
                reader.read_exact(&mut buf).await?;

                let ip = IpAddr::V6(Ipv6Addr::new(
                    u16::from_be_bytes([buf[0], buf[1]]),
                    u16::from_be_bytes([buf[2], buf[3]]),
                    u16::from_be_bytes([buf[4], buf[5]]),
                    u16::from_be_bytes([buf[6], buf[7]]),
                    u16::from_be_bytes([buf[8], buf[9]]),
                    u16::from_be_bytes([buf[10], buf[11]]),
                    u16::from_be_bytes([buf[12], buf[13]]),
                    u16::from_be_bytes([buf[14], buf[15]]),
                ));

                let port: u16 = u16::from_be_bytes([buf[16], buf[17]]);

                Some(NetAddr::new(Host::Ip(ip), port))
            }
            _ => {
                return Err(format!(
                    "ATYP must be {:#04x} or {:#04x} or {:#04x} but got {:#04x}",
                    ATYP_V4, ATYP_DOMAIN, ATYP_V6, buf[3]
                )
                .into());
            }
        };

        if addr.is_none() {
            return Err("couldn't resolve DST.ADDR".into());
        }

        // 4. Reply Socks5 Reply Message

        // Socks5 Reply Message
        // +----+-----+-------+------+----------+----------+
        // |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
        // +----+-----+-------+------+----------+----------+
        // | 1  |  1  | X'00' |  1   | Variable |    2     |
        // +----+-----+-------+------+----------+----------+
        writer
            .write(&[
                SOCKS_VERSION_V5,
                REPLY_SUCCEEDED,
                NOOP,
                ATYP_V4,
                NOOP,
                NOOP,
                NOOP,
                NOOP,
                NOOP,
                NOOP,
            ])
            .await?;

        let addr = addr.unwrap();

        self.proxy_address = Some(addr.clone());

        Ok(addr)
    }

    fn pack(&self, buf: Bytes) -> Result<Bytes> {
        let mut frame = BytesMut::new();

        let header_sent = &self.header_sent.lock().unwrap();

        if !header_sent.get() {
            let addr = self.proxy_address.as_ref().unwrap();
            let header = NetAddr::new(addr.host.clone(), addr.port);
            frame.put(header.as_bytes());

            header_sent.set(true);
        }

        frame.put(buf);

        Ok(frame.freeze())
    }

    fn unpack(&self, _buf: Bytes) -> Result<Bytes> {
        unimplemented!()
    }
}
