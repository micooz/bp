use super::proto::Protocol;
use crate::{net::address::NetAddr, utils::ToHex, Result, TcpStreamReader, TcpStreamWriter};
use async_trait::async_trait;
use bytes::Bytes;
use tokio::io::AsyncReadExt;

pub struct Plain {}

impl Plain {
    pub fn new() -> Self {
        Plain {}
    }
}

#[async_trait]
impl Protocol for Plain {
    fn get_name(&self) -> String {
        "plain".into()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        _writer: &mut TcpStreamWriter,
    ) -> Result<NetAddr> {
        let mut buf = vec![0u8; 0];
        reader.read(&mut buf).await?;

        log::debug!("{}", ToHex(buf));

        todo!()
    }

    async fn pack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }

    async fn unpack(&self, buf: Bytes) -> Result<Bytes> {
        Ok(buf)
    }
}
