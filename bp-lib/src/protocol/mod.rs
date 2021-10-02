use crate::{
    event::EventSender,
    net::{
        address::Address,
        io::{TcpStreamReader, TcpStreamWriter},
    },
    Result,
};
use async_trait::async_trait;
use bytes::Bytes;
use dyn_clone::DynClone;

mod erp;
mod http;
mod plain;
mod socks;
mod socks_http;
mod direct;

pub use erp::Erp;
pub use http::Http;
pub use plain::Plain;
pub use socks::Socks;
pub use socks_http::SocksHttp;
pub use direct::Direct;

#[async_trait]
pub trait Protocol: DynClone {
    fn get_name(&self) -> String;

    async fn resolve_proxy_address(
        &mut self,
        reader: &mut TcpStreamReader,
        writer: &mut TcpStreamWriter,
    ) -> Result<(Address, Option<Bytes>)>;

    fn set_proxy_address(&mut self, _addr: Address) {}

    fn get_proxy_address(&self) -> Option<Address> {
        unimplemented!()
    }

    async fn client_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn server_encode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn client_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;

    async fn server_decode(&mut self, reader: &mut TcpStreamReader, tx: EventSender) -> Result<()>;
}

dyn_clone::clone_trait_object!(Protocol);

pub type DynProtocol = Box<dyn Protocol + Send + Sync + 'static>;
