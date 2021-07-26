use super::proto::Protocol;
use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
};

// Socks4 Request Message
// +----+-----+----------+--------+----------+--------+
// |VER | CMD | DST.PORT | DST.IP | USER.ID  |  NULL  |
// +----+-----+----------+--------+----------+--------+
// | 1  |  1  |    2     |   4    | Variable |  X'00' |
// +----+-----+----------+--------+----------+--------+

// Socks4a Request Message
// +----+-----+----------+--------+----------+--------+------------+--------+
// |VER | CMD | DST.PORT | DST.IP | USER.ID  |  NULL  |  DST.ADDR  |  NULL  |
// +----+-----+----------+--------+----------+--------+------------+--------+
// | 1  |  1  |    2     |   4    | Variable |  X'00' |  Variable  |  X'00' |
// +----+-----+----------+--------+----------+--------+------------+--------+
//                        0.0.0.!0

// Socks4 Reply Message
// +----+-----+----------+--------+
// |VER | CMD | DST.PORT | DST.IP |
// +----+-----+----------+--------+
// | 1  |  1  |    2     |   4    |
// +----+-----+----------+--------+

// ------------------------------------------------------ //

// Socks5 Identifier Message
// +----+----------+----------+
// |VER | NMETHODS | METHODS  |
// +----+----------+----------+
// | 1  |    1     | 1 to 255 |
// +----+----------+----------+

// Socks5 Select Message
// +----+--------+
// |VER | METHOD |
// +----+--------+
// | 1  |   1    |
// +----+--------+

// Socks5 Initial negotiation(only when METHOD is 0x02)
// +----+------+----------+------+----------+
// |VER | ULEN |  UNAME   | PLEN |  PASSWD  |
// +----+------+----------+------+----------+
// | 1  |  1   | 1 to 255 |  1   | 1 to 255 |
// +----+------+----------+------+----------+

// Socks5 Request Message
// +----+-----+-------+------+----------+----------+
// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

// Socks5 Reply Message
// +----+-----+-------+------+----------+----------+
// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
// +----+-----+-------+------+----------+----------+
// | 1  |  1  | X'00' |  1   | Variable |    2     |
// +----+-----+-------+------+----------+----------+

// Socks5 UDP Request/Response
// +----+------+------+----------+----------+----------+
// |RSV | FRAG | ATYP | DST.ADDR | DST.PORT |   DATA   |
// +----+------+------+----------+----------+----------+
// | 2  |  1   |  1   | Variable |    2     | Variable |
// +----+------+------+----------+----------+----------+

// const NOOP: u8 = 0x00;
// const SOCKS_VERSION_V4: u8 = 0x04;
// const SOCKS_VERSION_V5: u8 = 0x05;
// const METHOD_NO_AUTH: u8 = 0x00;
// const METHOD_USERNAME_PASSWORD: u8 = 0x02;
// const METHOD_NOT_ACCEPTABLE: u8 = 0xff;

// const REQUEST_COMMAND_CONNECT: u8 = 0x01;
// const REQUEST_COMMAND_BIND: u8 = 0x02;
// const REQUEST_COMMAND_UDP: u8 = 0x03;

// const ATYP_V4: u8 = 0x01;
// const ATYP_DOMAIN: u8 = 0x03;
// const ATYP_V6: u8 = 0x04;

// const REPLY_GRANTED: u8 = 0x5a;
// const REPLY_SUCCEEDED: u8 = 0x00;
// const REPLY_FAILURE: u8 = 0x01;
// const REPLY_NOT_ALLOWED: u8 = 0x02;
// const REPLY_NETWORK_UNREACHABLE: u8 = 0x03;
// const REPLY_HOST_UNREACHABLE: u8 = 0x04;
// const REPLY_CONNECTION_REFUSED: u8 = 0x05;
// const REPLY_TTL_EXPIRED: u8 = 0x06;
// const REPLY_COMMAND_NOT_SUPPORTED: u8 = 0x07;
// const REPLY_ADDRESS_TYPE_NOT_SUPPORTED: u8 = 0x08;
// const REPLY_UNASSIGNED: u8 = 0xff;

#[derive(Debug)]
enum Stage {
    Init,
    // Negotiation,
    // Request,
    // Done,
}

#[derive(Debug)]
pub struct Socks5 {
    stage: Stage,
}

impl Socks5 {
    pub fn new() -> Self {
        Socks5 { stage: Stage::Init }
    }
}

#[async_trait]
impl Protocol for Socks5 {
    fn get_name(&self) -> String {
        "socks5".into()
    }

    async fn parse_proxy_address(
        &self,
        _reader: &mut ReadHalf<TcpStream>,
        writer: &mut WriteHalf<TcpStream>,
    ) -> Result<SocketAddr> {
        writer.write_u8(b'x').await?;

        let addr = "127.0.0.1:8080".parse().unwrap();

        Ok(addr)
    }

    async fn encode_data(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
