use bp_cli::{test_utils::run_bp, Options, ServiceContext};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::test(flavor = "multi_thread")]
async fn test_http_sniff() {
    // cmd_lib::init_builtin_logger();
    let opts = Options {
        client: true,
        ..Default::default()
    };

    let ServiceContext { bind_addr, .. } = run_bp(opts).await;

    let mut socket = TcpStream::connect(&bind_addr).await.unwrap();

    socket.write_all(include_bytes!("fixtures/http_req.bin")).await.unwrap();
    socket.flush().await.unwrap();

    let mut buf = BytesMut::with_capacity(15);
    let n = socket.read_buf(&mut buf).await.unwrap();

    let s = String::from_utf8(buf[0..n].to_vec()).unwrap();

    assert!(s.starts_with("HTTP/1.1 200 OK"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_https_sniff() {
    let opts = Options {
        client: true,
        ..Default::default()
    };

    let ServiceContext { bind_addr, .. } = run_bp(opts).await;

    let mut socket = TcpStream::connect(&bind_addr).await.unwrap();

    socket
        .write_all(include_bytes!("fixtures/https_client_hello.bin"))
        .await
        .unwrap();

    socket.flush().await.unwrap();

    let mut buf = BytesMut::with_capacity(15);
    let _n = socket.read_buf(&mut buf).await.unwrap();

    let server_hello_partial = &[0x16, 0x03, 0x03];

    assert_eq!(&buf[0..3], server_hello_partial);
}
