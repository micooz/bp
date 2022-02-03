use bp_core::{ClientOptions, ServerOptions};
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};

#[tokio::test(flavor = "multi_thread")]
async fn test_proxy_white_list() {
    let resp = run_all(
        ClientOptions {
            proxy_white_list: Some("tests/fixtures/proxy_white_list.txt".to_string()),
            ..Default::default()
        },
        ServerOptions::default(),
        None,
    )
    .await;

    let TestResponse { bind_addr, .. } = resp;

    assert!(run_fun!(curl --socks5 $bind_addr cn.bing.com).is_ok());
    assert!(run_fun!(curl --socks5 $bind_addr www.baidu.com).is_ok());
    assert!(run_fun!(curl --socks5 $bind_addr example.com).is_ok());
}
