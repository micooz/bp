use bp_core::Options;
use cmd_lib::run_fun;
use e2e::run_all::{run_all, TestResponse};
use rcgen::generate_simple_self_signed;

#[tokio::test(flavor = "multi_thread")]
async fn test_quic() {
    let (certificate, privatekey) = generate_cert_and_key().await;

    let resp = run_all(Options {
        quic: true,
        certificate: Some(certificate),
        privatekey: Some(privatekey),
        ..Default::default()
    })
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

async fn generate_cert_and_key() -> (String, String) {
    let subject_alt_names = vec!["localhost".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).unwrap();

    let certificate = String::from("tests/tmp/cert.der");
    let privatekey = String::from("tests/tmp/key.der");

    tokio::fs::write(&certificate, cert.serialize_der().unwrap())
        .await
        .unwrap();

    tokio::fs::write(&privatekey, cert.serialize_private_key_der())
        .await
        .unwrap();

    (certificate, privatekey)
}
