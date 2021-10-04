use crate::{
    event::EventSender,
    net::{address::Address, socket},
    protocol::{socks, Http, Protocol, Socks},
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;

pub struct SocksHttp {
    socks_bind_addr: Option<Address>,
    proxy_address: Option<Address>,
}

impl SocksHttp {
    pub fn new(socks_bind_addr: Option<Address>) -> Self {
        Self {
            socks_bind_addr,
            proxy_address: None,
        }
    }
}

impl Clone for SocksHttp {
    fn clone(&self) -> Self {
        Self {
            socks_bind_addr: self.socks_bind_addr.clone(),
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

    async fn resolve_proxy_address(&mut self, socket: &socket::Socket) -> Result<(Address, Option<Bytes>)> {
        if socket.is_tcp() {
            let reader = socket.tcp_reader();
            let mut reader = reader.lock().await;

            let buf = reader.read_exact(1).await?;
            reader.cache(buf.clone());

            // TODO: improve this assertion
            if buf[0] == socks::SOCKS_VERSION_V5 {
                let mut socks = Socks::new(self.socks_bind_addr.clone());
                socks.resolve_proxy_address(socket).await
            } else {
                let mut http = Http::new();
                http.resolve_proxy_address(socket).await
            }
        } else {
            // Socks UDP packet goes here
            let packet = socket.udp_packet().unwrap();
            let atyp = packet[3];

            // TODO: improve this assertion
            if atyp == socks::ATYP_V4 || atyp == socks::ATYP_V6 || atyp == socks::ATYP_DOMAIN {
                let mut socks = Socks::new(self.socks_bind_addr.clone());
                socks.resolve_proxy_address(socket).await
            } else {
                Err("invalid socks5 udp packet".into())
            }
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
