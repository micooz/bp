use bp_lib::net::io::TcpStreamWriter;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Context {
    pub peer_addr: SocketAddr,
    pub writer: Arc<Mutex<TcpStreamWriter>>,
}
