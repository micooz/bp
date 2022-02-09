use bp_core::{ClientOptions, ServerOptions};
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};

#[tokio::test(flavor = "multi_thread")]
async fn test_acl() {
    let resp = run_all(
        ClientOptions {
            acl: Some("tests/fixtures/acl.txt".to_string()),
            ..Default::default()
        },
        ServerOptions::default(),
        None,
    )
    .await;

    let TestResponse { bind_addr, .. } = resp;

    assert!(run_fun!(curl --socks5 $bind_addr cn.bing.com).is_ok());
    assert!(run_fun!(curl --socks5 $bind_addr www.baidu.com).is_ok());
}
