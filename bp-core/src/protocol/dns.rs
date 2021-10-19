use crate::{
    event::{Event, EventSender},
    net::socket::Socket,
    protocol::{Protocol, ResolvedResult},
    Address, Result,
};
use async_trait::async_trait;
use simple_dns::Packet;

#[derive(Default, Clone)]
pub struct Dns {
    resolved_result: Option<ResolvedResult>,
}

impl Dns {
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
            protocol: self.get_name(),
            // TODO: send dns server addr to bp server is unnecessary
            // NOTE: in order to make it work on relay mode(not set --server-bind), we must pass this value (currently).
            address: Address::default(),
            pending_buf: Some(buf),
        });

        Ok(())
    }

    async fn client_encode(&mut self, socket: &Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_some().await?;
        tx.send(Event::ClientEncodeDone(buf)).await?;
        Ok(())
    }

    async fn server_encode(&mut self, socket: &Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_some().await?;
        tx.send(Event::ServerEncodeDone(buf)).await?;
        Ok(())
    }

    async fn client_decode(&mut self, socket: &Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_some().await?;
        tx.send(Event::ClientDecodeDone(buf)).await?;
        Ok(())
    }

    async fn server_decode(&mut self, socket: &Socket, tx: EventSender) -> Result<()> {
        let buf = socket.read_some().await?;
        tx.send(Event::ServerDecodeDone(buf)).await?;
        Ok(())
    }
}
