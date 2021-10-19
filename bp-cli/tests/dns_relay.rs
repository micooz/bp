use bp_cli::test_utils::run_bp;
use bp_core::Options;
use bp_test::send_recv::udp_oneshot;

#[tokio::test(flavor = "multi_thread")]
async fn test_dns_relay_without_server() {
    let client_opts = Options {
        client: true,
        dns_over_tcp: true,
        ..Options::default()
    };

    let client = run_bp(client_opts).await;

    let buf = udp_oneshot(&client.bind_addr, include_bytes!("fixtures/normal_dns_query.bin")).await;

    assert!(!buf.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_dns_relay_with_server() {
    let server_opts = Options {
        server: true,
        key: Some("key".to_string()),
        enable_udp: true,
        ..Options::default()
    };

    let server = run_bp(server_opts).await;

    let client_opts = Options {
        client: true,
        key: Some("key".to_string()),
        server_bind: Some(server.bind_addr.clone()),
        dns_over_tcp: true,
        ..Options::default()
    };

    let client = run_bp(client_opts).await;

    let buf = udp_oneshot(&client.bind_addr, include_bytes!("fixtures/normal_dns_query.bin")).await;

    assert!(!buf.is_empty());
}
