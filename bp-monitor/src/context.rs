use bp_lib::net::socket::Socket;
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug)]
pub struct Context {
    pub peer_addr: SocketAddr,
    pub socket: Arc<Socket>,
}
