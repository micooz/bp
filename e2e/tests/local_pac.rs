use bp_core::{Address, ClientOptions, Options};
use cmd_lib::run_fun;
use e2e::runner::run_bp_custom;

#[tokio::test(flavor = "multi_thread")]
async fn test_pac() {
    let pac_bind: Address = "127.0.0.1:8000".parse().unwrap();

    let opts = Options::Client(ClientOptions {
        pac_bind: Some(pac_bind.clone()),
        acl: Some("tests/fixtures/acl.txt".to_string()),
        ..Default::default()
    });

    run_bp_custom(opts, Some("127.0.0.1"), Some(3000)).await;

    let pac_bind = pac_bind.to_string();

    insta::assert_snapshot!("proxy.pac", run_fun!(curl $pac_bind/proxy.pac).unwrap());
    insta::assert_snapshot!("proxy.pac2", run_fun!(curl $pac_bind/proxy.pac?xxx).unwrap());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pac_with_proxy() {
    let pac_bind: Address = "127.0.0.1:8001".parse().unwrap();
    let pac_proxy: Address = "anyproxy:1080".parse().unwrap();

    let opts = Options::Client(ClientOptions {
        pac_bind: Some(pac_bind.clone()),
        pac_proxy: Some(pac_proxy),
        acl: Some("tests/fixtures/acl.txt".to_string()),
        ..Default::default()
    });

    run_bp_custom(opts, Some("127.0.0.1"), Some(3001)).await;

    let pac_bind = pac_bind.to_string();

    insta::assert_snapshot!("proxy.pac3", run_fun!(curl $pac_bind/proxy.pac?xxx).unwrap());
}
