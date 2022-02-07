use bp_cli::commands::test;
use e2e::http_server::{run_http_mock_server, HttpServerContext};

#[tokio::test(flavor = "multi_thread")]
async fn test_test() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let config = "tests/fixtures/test_config.json";
    let resp = test::http_request_via_client(config, http_addr.into()).await.unwrap();

    assert!(resp.ends_with(http_resp));
}
