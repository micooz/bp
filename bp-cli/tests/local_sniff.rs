use bp_cli::{test_utils::run_bp, Options};
use bp_core::net::service::StartupInfo;
use bp_test::send_recv::tcp_oneshot;

#[tokio::test(flavor = "multi_thread")]
async fn test_http_sniff() {
    // cmd_lib::init_builtin_logger();
    let opts = Options {
        client: true,
        ..Default::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(&bind_addr, include_bytes!("fixtures/http_req.bin")).await;
    let resp = String::from_utf8(buf).unwrap();

    assert!(resp.starts_with("HTTP/1.1 200 OK"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_https_sniff() {
    let opts = Options {
        client: true,
        ..Default::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(&bind_addr, include_bytes!("fixtures/https_client_hello.bin")).await;
    let server_hello_partial = &[0x16, 0x03, 0x03];

    assert_eq!(&buf[0..3], server_hello_partial);
}
