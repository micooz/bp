use bp_cli::test_utils::run_bp;
use bp_core::net::service::StartupInfo;
use bp_core::Options;
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

    assert!(!buf.is_empty());
}
