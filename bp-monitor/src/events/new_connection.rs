use std::net::SocketAddr;

use serde::Serialize;

use super::Event;

#[derive(Serialize)]
pub struct NewConnection {
    pub name: &'static str,
    pub peer_addr: SocketAddr,
    pub live_cnt: usize,
    pub total_cnt: usize,
}

impl Event for NewConnection {}
