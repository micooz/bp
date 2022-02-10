use bp_core::{ClientOptions, Options, StartupInfo};
use cmd_lib::run_fun;
use e2e::{
    http_server::{run_http_mock_server, HttpServerContext},
    oneshot::{tcp_oneshot, udp_oneshot},
    runner::run_bp,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;
    let bind_addr = bind_addr.to_string();

    assert_eq!(run_fun!(curl --socks5 $bind_addr $http_addr).unwrap(), http_resp);
    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5_udp() {
    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = udp_oneshot(bind_addr, include_bytes!("fixtures/socks5_dns_query.bin")).await;

    // TODO: improve this assertion
    assert_eq!(buf[0..7], include_bytes!("fixtures/dns_resp.bin")[0..7]);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;
    let bind_addr = bind_addr.to_string();

    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), http_resp);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http_with_auth() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let auth = "user:pass";

    let opts = Options::Client(ClientOptions {
        with_basic_auth: Some(auth.parse().unwrap()),
        ..Default::default()
    });

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;
    let bind_addr = bind_addr.to_string();

    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), "");
    assert_eq!(run_fun!(curl -u $auth -x $bind_addr $http_addr).unwrap(), http_resp);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pin_dest_addr() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let opts = Options::Client(ClientOptions {
        pin_dest_addr: Some(http_addr.into()),
        ..Default::default()
    });

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(bind_addr, include_bytes!("fixtures/http_req_mock.bin")).await;
    let resp = String::from_utf8(buf).unwrap();

    assert!(resp.contains(http_resp));
}
