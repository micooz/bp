use bp_core::{ClientOptions, EncryptionMethod, ServerOptions};
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};

#[tokio::test(flavor = "multi_thread")]
async fn test_plain() {
    let resp = run_all(
        ClientOptions {
            encryption: EncryptionMethod::Plain,
            ..Default::default()
        },
        ServerOptions {
            encryption: EncryptionMethod::Plain,
            ..Default::default()
        },
        None,
    )
    .await;

    let TestResponse {
        bind_addr,
        http_addr,
        http_resp,
    } = resp;

    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_erp() {
    let resp = run_all(
        ClientOptions {
            encryption: EncryptionMethod::EncryptRandomPadding,
            ..Default::default()
        },
        ServerOptions {
            encryption: EncryptionMethod::EncryptRandomPadding,
            ..Default::default()
        },
        None,
    )
    .await;

    let TestResponse {
        bind_addr,
        http_addr,
        http_resp,
    } = resp;

    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}
