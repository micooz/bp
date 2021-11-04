use std::{net::SocketAddr, sync::Arc};

use bp_core::Socket;

#[derive(Debug)]
pub struct Context {
    pub peer_addr: SocketAddr,
    pub socket: Arc<Socket>,
}
