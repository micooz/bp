use bp_core::{Options, StartupInfo};
use e2e::{oneshot::tcp_oneshot, run_bp::run_bp};

#[tokio::test(flavor = "multi_thread")]
async fn test_http_sniff() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts, None).await;

    let buf = tcp_oneshot(bind_addr, include_bytes!("fixtures/http_req.bin")).await;
    let resp = String::from_utf8(buf).unwrap();

    assert!(resp.starts_with("HTTP/1.1 200 OK"));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_https_sniff() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts, None).await;

    let buf = tcp_oneshot(bind_addr, include_bytes!("fixtures/https_client_hello.bin")).await;

    assert!(!buf.is_empty());
}