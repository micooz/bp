use crate::{
    event::EventSender,
    net::{address::Address, socket},
    protocol::{socks, Http, Protocol, Socks},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;

#[derive(Clone, Default)]
pub struct Universal {
    socks_bind_addr: Option<Address>,
    proxy_address: Option<Address>,
}

impl Universal {
    pub fn new(socks_bind_addr: Option<Address>) -> Self {
        Self {
            socks_bind_addr,
            proxy_address: None,
        }
    }
}

#[async_trait]
impl Protocol for Universal {
    fn get_name(&self) -> String {
        "universal".into()
    }

    fn set_proxy_address(&mut self, addr: Address) {
        self.proxy_address = Some(addr);
    }

    fn get_proxy_address(&self) -> Option<Address> {
        self.proxy_address.clone()
    }

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<(Address, Option<Bytes>)> {
        if socket.is_tcp() {
            let buf = socket.read_exact(1).await?;
            socket.cache(buf.clone()).await;

            // TODO: improve this assertion
            if buf[0] == socks::SOCKS_VERSION_V5 {
                let mut socks = Socks::new(self.socks_bind_addr.clone());
                return socks.resolve_proxy_address(socket).await;
            }

            // TODO: HTTP check
            let mut http = Http::default();
            http.resolve_proxy_address(socket).await

            // TODO: HTTPS check
        } else {
            // Socks UDP packet check
            let packet = socket.read_exact(4).await?;
            socket.cache(packet.clone()).await;

            let atyp = packet[3];

            // TODO: improve this assertion
            if atyp == socks::ATYP_V4 || atyp == socks::ATYP_V6 || atyp == socks::ATYP_DOMAIN {
                let mut socks = Socks::new(self.socks_bind_addr.clone());
                return socks.resolve_proxy_address(socket).await;
            }

            // TODO: DNS UDP packet check

            // TODO: Unknown UDP packet

            Err("invalid socks5 udp packet".into())
        }
    }

    async fn client_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_encode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn client_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }

    async fn server_decode(&mut self, _socket: &socket::Socket, _tx: EventSender) -> Result<()> {
        unimplemented!()
    }
}
