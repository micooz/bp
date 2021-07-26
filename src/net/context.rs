use crate::options::Options;
use std::net::SocketAddr;

/// The context shared inside Connection
pub struct Context {
    pub opts: Option<Options>,
    pub proxy_address: Option<SocketAddr>,
    pub peer_address: Option<SocketAddr>,
}

impl Context {
    pub fn new(opts: Options) -> Self {
        Context {
            opts: Some(opts),
            proxy_address: None,
            peer_address: None,
        }
    }
}
