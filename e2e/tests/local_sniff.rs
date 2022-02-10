use bp_core::{ClientOptions, Options, StartupInfo};
use e2e::{oneshot::tcp_oneshot, runner::run_bp};

#[tokio::test(flavor = "multi_thread")]
async fn test_http_sniff() {
    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(bind_addr, include_bytes!("fixtures/http_req.bin")).await;
    let resp = String::from_utf8(buf).unwrap();

    assert!(resp.starts_with("HTTP/1.1 200 OK"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_https_sniff() {
    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(bind_addr, include_bytes!("fixtures/https_client_hello.bin")).await;

    assert!(!buf.is_empty());
}
