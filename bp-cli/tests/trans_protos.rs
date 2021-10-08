use bp_cli::test_utils::run_bp;
use bp_cli::Options;
use bp_core::TransportProtocol;
use bp_test::http_server::{run_http_mock_server, HttpServerContext};
use cmd_lib::run_fun;

#[tokio::test(flavor = "multi_thread")]
async fn test_transport_plain() {
    run_test(TransportProtocol::Plain).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_transport_erp() {
    run_test(TransportProtocol::EncryptRandomPadding).await;
}

async fn run_test(protocol: TransportProtocol) {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    // start server
    let server_opts = Options {
        server: true,
        key: Some("key".to_string()),
        protocol: protocol.clone(),
        ..Default::default()
    };
    let server = run_bp(server_opts).await;

    // start client
    let client_opts = Options {
        client: true,
        key: Some("key".to_string()),
        server_bind: Some(server.bind_addr.clone()),
        protocol,
        ..Default::default()
    };
    let client = run_bp(client_opts).await;

    let bind_addr = client.bind_addr;

    assert_eq!(run_fun!(curl --socks5 $bind_addr $http_addr).unwrap(), http_resp);
    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), http_resp);
}
