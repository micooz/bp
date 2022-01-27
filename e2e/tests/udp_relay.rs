use bp_core::Options;
use e2e::{oneshot::udp_oneshot, run_bp::run_bp};

#[tokio::test(flavor = "multi_thread")]
async fn test_udp_relay_without_server() {
    let client_opts = Options {
        client: true,
        ..Options::default()
    };

    let client = run_bp(client_opts, None).await;

    let buf = udp_oneshot(client.bind_addr, include_bytes!("fixtures/normal_dns_query.bin")).await;

    assert!(!buf.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_udp_relay_with_server() {
    run_test(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_udp_over_tcp() {
    run_test(true).await;
}

async fn run_test(udp_over_tcp: bool) {
    let server_opts = Options {
        server: true,
        key: Some("key".to_string()),
        ..Options::default()
    };

    let server = run_bp(server_opts, None).await;

    let client_opts = Options {
        client: true,
        key: Some("key".to_string()),
        server_bind: Some(server.bind_addr.into()),
        udp_over_tcp,
        ..Options::default()
    };

    let client = run_bp(client_opts, None).await;

    let buf = udp_oneshot(client.bind_addr, include_bytes!("fixtures/normal_dns_query.bin")).await;

    assert!(!buf.is_empty());
}
