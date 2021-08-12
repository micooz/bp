use crate::{
    event::EventSender,
    net::{Address, TcpStreamReader, TcpStreamWriter},
    protocol::{socks::SOCKS_VERSION_V5, Http, Protocol, Socks},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;

pub struct SocksHttp {
    proxy_address: Option<Address>,
}

impl SocksHttp {
    pub fn new() -> Self {
        Self { proxy_address: None }
    }
}

impl Clone for SocksHttp {
    fn clone(&self) -> Self {
        Self {
            proxy_address: self.proxy_address.clone(),
        }
    }
}

#[async_trait]
impl Protocol for SocksHttp {
    fn get_name(&self) -> String {
        "socks_http".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)> {
        let buf = reader.read_exact(1).await?;
        reader.cache(&buf);

        // TODO: improve this assertion
        if buf[0] == SOCKS_VERSION_V5 {
            let mut socks = Socks::new();
            socks.resolve_proxy_address(reader, writer).await
        } else {
            let mut http = Http::new();
            http.resolve_proxy_address(reader, writer).await
        }
    }

    async fn client_encode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _reader: &mut TcpStreamReader, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }
}
