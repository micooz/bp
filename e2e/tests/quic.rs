use bp_core::{utils::tls::TLS, Options};
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};

#[tokio::test(flavor = "multi_thread")]
async fn test_quic() {
    let cert_path = String::from("tests/tmp/cert.der");
    let key_path = String::from("tests/tmp/key.der");

    TLS::gen_cert_and_key(vec!["localhost".to_string()], &cert_path, &key_path)
        .await
        .unwrap();

    let resp = run_all(
        Options {
            quic: true,
            tls_cert: Some(cert_path),
            tls_key: Some(key_path),
            ..Default::default()
        },
        Some("localhost"),
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
