use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use dns_parser::Packet;

use crate::{
    net::socket::Socket,
    protocol::{Protocol, ProtocolType, ResolvedResult},
    Address,
};

#[derive(Clone)]
pub struct Dns {
    dns_server: Address,
    resolved_result: Option<ResolvedResult>,
}

impl Dns {
    pub fn new(dns_server: Address) -> Self {
        Self {
            dns_server,
            resolved_result: None,
        }
    }

    pub fn check_dns_query(buf: &[u8]) -> bool {
        Packet::parse(buf).is_ok()
    }
}

#[async_trait]
impl Protocol for Dns {
    fn get_name(&self) -> String {
        "dns".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> Option<ResolvedResult> {
        self.resolved_result.clone()
    }

    async fn resolve_dest_addr(&mut self, socket: &Socket) -> Result<()> {
        let buf = socket.read_some().await?;

        self.set_resolved_result(ResolvedResult {
            protocol: ProtocolType::Dns,
            // TODO: send dns server addr to bp server is unnecessary
            // NOTE: in order to make it work on relay mode(not set --server-bind), we must pass this value (currently).
            address: self.dns_server.clone(),
            pending_buf: Some(buf),
        });

        Ok(())
    }

    async fn client_encode(&mut self, socket: &Socket) -> Result<Bytes> {
        socket.read_some().await
    }

    async fn server_encode(&mut self, socket: &Socket) -> Result<Bytes> {
        socket.read_some().await
    }

    async fn client_decode(&mut self, socket: &Socket) -> Result<Bytes> {
        socket.read_some().await
    }

    async fn server_decode(&mut self, socket: &Socket) -> Result<Bytes> {
        socket.read_some().await
    }
}
