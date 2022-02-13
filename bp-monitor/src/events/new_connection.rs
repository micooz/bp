use std::net::SocketAddr;

use serde::Serialize;

use super::Event;

#[derive(Serialize)]
pub struct NewConnectionEvent {
    pub peer_addr: SocketAddr,
}

impl Event for NewConnectionEvent {
    fn name() -> String {
        "NewConnectionEvent".to_string()
    }
}
