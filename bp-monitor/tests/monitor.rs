use std::{net::SocketAddr, sync::Arc};

use bp_monitor::{events, Monitor, Subscriber};
use tokio::net::UdpSocket;

#[tokio::test(flavor = "multi_thread")]
async fn test_monitor() {
    let peer_addr: SocketAddr = "127.0.0.1:6666".parse().unwrap();
    let socket = UdpSocket::bind(peer_addr).await.unwrap();
    let subscriber = Subscriber::Udp((Arc::new(socket), peer_addr));

    let mut monitor = Monitor::default();
    monitor.add_subscriber(subscriber).unwrap();
    monitor.log(events::NewConnectionEvent { peer_addr });
}
