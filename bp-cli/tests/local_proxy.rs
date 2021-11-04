use bp_cli::test_utils::run_bp;
use bp_core::{Options, StartupInfo};
use bp_test::{
    http_server::{run_http_mock_server, HttpServerContext},
    send_recv::udp_oneshot,
};
use cmd_lib::run_fun;

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options {
        client: true,
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;
    let bind_addr = bind_addr.as_string();

    assert_eq!(run_fun!(curl --socks5 $bind_addr $http_addr).unwrap(), http_resp);
    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5_udp() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = udp_oneshot(&bind_addr, include_bytes!("fixtures/socks5_dns_query.bin")).await;

    // TODO: improve this assertion
    assert_eq!(buf[0..7], include_bytes!("fixtures/dns_resp.bin")[0..7]);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options {
        client: true,
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;
    let bind_addr = bind_addr.as_string();

    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), http_resp);
}
