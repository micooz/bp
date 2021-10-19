use bp_cli::test_utils::run_bp;
use bp_core::Options;
use cmd_lib::run_fun;

#[tokio::test(flavor = "multi_thread")]
async fn test_proxy_white_list() {
    run_test("cn.bing.com").await;
    run_test("www.baidu.com").await;
    run_test("example.com").await;
}

async fn run_test(dest_addr: &str) {
    // start server
    let server_opts = Options {
        server: true,
        key: Some("key".to_string()),
        ..Options::default()
    };
    let server = run_bp(server_opts).await;

    // start client
    let client_opts = Options {
        client: true,
        key: Some("key".to_string()),
        server_bind: Some(server.bind_addr.clone()),
        proxy_white_list: Some("tests/fixtures/proxy_white_list.txt".to_string()),
        ..Options::default()
    };
    let client = run_bp(client_opts).await;

    let bind_addr = client.bind_addr.as_string();

    assert!(run_fun!(curl --socks5-hostname $bind_addr $dest_addr).is_ok());
}
