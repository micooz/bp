use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    net::socket::Socket,
    protos::{Protocol, ResolvedResult},
};

#[derive(Default, Clone)]
pub struct Direct {
    resolved_result: Option<ResolvedResult>,
}

#[async_trait]
impl Protocol for Direct {
    fn get_name(&self) -> String {
        "direct".into()
    }

    fn set_resolved_result(&mut self, res: ResolvedResult) {
        self.resolved_result = Some(res);
    }

    fn get_resolved_result(&self) -> &ResolvedResult {
        self.resolved_result.as_ref().unwrap()
    }

    async fn resolve_dest_addr(&mut self, _socket: &Socket) -> Result<&ResolvedResult> {
        panic!("direct protocol cannot be used on inbound")
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
