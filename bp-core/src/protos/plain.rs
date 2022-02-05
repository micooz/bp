use anyhow::Result;
use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};

use crate::{
    net::{address::Address, socket},
    protos::{Protocol, ProtocolType, ResolvedResult},
    Socket,
};

/// # Protocol
/// +------+----------+----------+-------------+
/// | ATYP | DST.ADDR | DST.PORT |    Data     |
/// +------+----------+----------+-------------+
/// |  1   | Variable |    2     |  Variable   |
/// +------+----------+----------+-------------+
#[derive(Clone, Default)]
pub struct Plain {
    header_sent: bool,
    resolved_result: Option<ResolvedResult>,
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> &ResolvedResult {
        self.resolved_result.as_ref().unwrap()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<&ResolvedResult> {
        let address = Address::from_socket(socket).await?;

        self.set_resolved_result(ResolvedResult {
            protocol: ProtocolType::Plain,
            address,
            pending_buf: None,
        });

        Ok(self.get_resolved_result())
    }

    async fn client_encode(&mut self, socket: &socket::Socket) -> Result<Bytes> {
        let mut frame = BytesMut::new();

        if !self.header_sent {
            let resolved = self.get_resolved_result();
            frame.put(resolved.address.as_bytes());
            self.header_sent = true;
        }

        let buf = socket.read_some().await?;
        frame.put(buf);

        Ok(frame.freeze())
    }

    async fn server_encode(&mut self, socket: &socket::Socket) -> Result<Bytes> {
        socket.read_some().await
    }

    async fn client_decode(&mut self, socket: &socket::Socket) -> Result<Bytes> {
        socket.read_some().await
    }

    async fn server_decode(&mut self, socket: &socket::Socket) -> Result<Bytes> {
        socket.read_some().await
    }
}
