use crate::{
    event, net,
    net::socket,
    protocol,
    protocol::{http::Http, socks::Socks},
    Result,
};
use async_trait::async_trait;

#[derive(Clone, Default)]
pub struct Universal {
    socks_bind_addr: Option<net::Address>,
    proxy_address: Option<net::Address>,
}

impl Universal {
    pub fn new(socks_bind_addr: Option<net::Address>) -> Self {
        Self {
            socks_bind_addr,
            proxy_address: None,
        }
    }
}

#[async_trait]
impl protocol::Protocol for Universal {
    fn get_name(&self) -> String {
        "universal".into()
    }

    fn set_proxy_address(&mut self, addr: net::Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<net::Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<protocol::ResolvedResult> {
        log::debug!("use [socks] to detect proxy address...");

        // SOCKS
        let mut socks = Socks::new(self.socks_bind_addr.clone());
        let res = socks.resolve_proxy_address(socket).await;

        if res.is_ok() {
            return res;
        }

        log::debug!("use [socks] to detect proxy address...failed due to: {}", res.unwrap_err());

        socket.restore().await;

        log::debug!("use [http] to detect proxy address...");

        // HTTP
        let mut http = Http::default();
        let res = http.resolve_proxy_address(socket).await;

        if res.is_ok() {
            return res;
        }

        log::debug!("use [http] to detect proxy address...failed due to: {}", res.unwrap_err());

        Err("cannot resolve proxy address ".into())
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
