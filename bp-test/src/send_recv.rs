use bp_core::{net::Address, utils::net::create_udp_client_with_random_port};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn tcp_oneshot(bind_addr: &Address, buf: &[u8]) -> Vec<u8> {
    let mut socket = TcpStream::connect(bind_addr.as_socket_addr()).await.unwrap();

    socket.write_all(buf).await.unwrap();
    socket.flush().await.unwrap();

    let mut buf = [0; 2048];
    let n = socket.read(&mut buf).await.unwrap();

    buf[0..n].to_vec()
}

pub async fn udp_oneshot(bind_addr: &Address, buf: &[u8]) -> Vec<u8> {
    let socket = create_udp_client_with_random_port().await.unwrap();

    socket.send_to(buf, bind_addr.as_socket_addr()).await.unwrap();

    let mut buf = vec![0u8; 1500];
    let n = socket.recv(&mut buf).await.unwrap();
    let buf = &buf[..n];

    buf[0..n].to_vec()
}
