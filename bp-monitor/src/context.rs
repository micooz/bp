use bp_lib::TcpStreamWriter;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

pub struct Context {
    pub peer_addr: SocketAddr,
    pub writer: Arc<Mutex<TcpStreamWriter>>,
    // pub reader: TcpStreamReader,
}
