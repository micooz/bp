use crate::{
    event,
    net::{self, socket},
    protocol::{Protocol, ResolvedResult},
    utils, Result,
};
use async_trait::async_trait;
use bytes::Buf;

#[derive(Clone, Default)]
pub struct Https {
    proxy_address: Option<net::Address>,
}

#[async_trait]
impl Protocol for Https {
    fn get_name(&self) -> String {
        "https".into()
    }

    fn set_proxy_address(&mut self, addr: net::Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<net::Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<ResolvedResult> {
        let content_type = socket.read_exact(1).await?;

        if content_type[0] != 0x16 {
            return Err(format!(
                "Content Type must be Handshake (0x16), but got {:#04x}",
                content_type[0]
            )
            .into());
        }

        let version = socket.read_exact(2).await?;

        if version[0] != 0x03 && version[1] != 0x01 {
            return Err(format!(
                "Version must be TLS 1.0 (0x0301), but got {}",
                utils::fmt::ToHex(version.to_vec())
            )
            .into());
        }

        let mut len_buf = socket.read_exact(2).await?;
        let mut handshake_buf = socket.read_exact(len_buf.get_u16() as usize).await?;

        socket.restore().await;

        let handshake_type = handshake_buf.get_u8();

        if handshake_type != 0x01 {
            return Err(format!(
                "Handshake Type must be Client Hello (0x01), but got {:#04x}",
                handshake_type
            )
            .into());
        }

        // skip Length(3 bytes)
        handshake_buf.advance(3);

        let version = handshake_buf.slice(0..2);

        if version[0] != 0x03 && version[1] != 0x03 {
            return Err(format!(
                "Version must be TLS 1.2 (0x0303), but got {}",
                utils::fmt::ToHex(version.to_vec())
            )
            .into());
        }

        // skip Version(2 bytes) and Random(32 bytes)
        handshake_buf.advance(34);

        let session_id_len = handshake_buf.get_u8();

        // skip Session ID(n bytes)
        handshake_buf.advance(session_id_len as usize);

        let cipher_suites_len = handshake_buf.get_u16();

        // skip Cipher Suites(n bytes)
        handshake_buf.advance(cipher_suites_len as usize);

        let compression_methods_len = handshake_buf.get_u8();

        // skip Compression Methods(n bytes)
        handshake_buf.advance(compression_methods_len as usize);

        let extensions_len = handshake_buf.get_u16() as usize;
        let ext_buf = handshake_buf;

        if extensions_len != ext_buf.len() {
            return Err(format!(
                "Extension Length({}) mismatch the remaining buffer size({})",
                extensions_len,
                ext_buf.len()
            )
            .into());
        }

        // find Extension: server_name
        let mut cur = 0;

        loop {
            let ext_type = ext_buf.slice(cur..cur + 2);
            let ext_len = ext_buf[cur + 3] as usize;

            // server_name extension
            if ext_type[0] == 0x00 && ext_type[1] == 0x00 {
                let cur = cur + 4;

                let mut sni = ext_buf.slice(cur..cur + ext_len);
                let _server_name_list_len = sni.get_u16();
                let _server_name_type = sni.get_u8();
                let server_name_len = sni.get_u16() as usize;
                let server_name = String::from_utf8(sni.slice(0..server_name_len).to_vec())?;

                return Ok(ResolvedResult {
                    protocol: self.get_name(),
                    address: net::Address::new(net::address::Host::Name(server_name), 443),
                    pending_buf: None,
                });
            }

            // skip Type and Length
            cur += 4;
            cur += ext_len;

            if cur >= extensions_len {
                break;
            }
        }

        Err("server_name Extension not found".into())
    }

    async fn client_encode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &socket::Socket, _tx: event::EventSender) -> Result<()> {
        unimplemented!()
    }
}
