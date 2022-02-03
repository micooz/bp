use bp_core::{ClientOptions, Options, StartupInfo};
use cmd_lib::run_fun;
use e2e::{
    http_server::{run_http_mock_server, HttpServerContext},
    oneshot::udp_oneshot,
    run_bp::run_bp,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts, None).await;
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

    let StartupInfo { bind_addr, .. } = run_bp(opts, Some("127.0.0.1")).await;

    let buf = udp_oneshot(bind_addr, include_bytes!("fixtures/socks5_dns_query.bin")).await;

    // TODO: improve this assertion
    assert_eq!(buf[0..7], include_bytes!("fixtures/dns_resp.bin")[0..7]);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options::Client(ClientOptions::default());

    let StartupInfo { bind_addr, .. } = run_bp(opts, None).await;
    let bind_addr = bind_addr.to_string();

    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), http_resp);
}
