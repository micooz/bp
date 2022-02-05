use std::sync::Once;

use bp_core::{utils::tls, ClientOptions, ServerOptions};
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};

static INIT: Once = Once::new();

const HOSTNAME: &str = "localhost";
const CERT_PATH: &str = "tests/tmp/cert.der";
const KEY_PATH: &str = "tests/tmp/key.der";

pub fn initialize() {
    INIT.call_once(|| {
        tls::generate_cert_and_key(vec![HOSTNAME.to_string()], &CERT_PATH, &KEY_PATH).unwrap();
    });
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tls() {
    let TestResponse {
        bind_addr,
        http_addr,
        http_resp,
    } = run_test(true, false).await;

    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_quic() {
    let TestResponse {
        bind_addr,
        http_addr,
        http_resp,
    } = run_test(false, true).await;

    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

async fn run_test(tls: bool, quic: bool) -> TestResponse {
    initialize();

    run_all(
        ClientOptions {
            tls,
            quic,
            tls_cert: Some(CERT_PATH.to_string()),
            ..Default::default()
        },
        ServerOptions {
            tls,
            quic,
            tls_cert: Some(CERT_PATH.to_string()),
            tls_key: Some(KEY_PATH.to_string()),
            ..Default::default()
        },
        Some(HOSTNAME),
    )
    .await
}
