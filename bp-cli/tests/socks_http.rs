use bp_cli::test_utils::run_bp;
use bp_cli::{Options, ServiceContext};
use bp_test::http_server::{run_http_mock_server, HttpServerContext};
use bp_test::send_recv::udp_oneshot;
use cmd_lib::run_fun;

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5() {
    // cmd_lib::init_builtin_logger();
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options {
        client: true,
        ..Default::default()
    };

    let ServiceContext { bind_addr, .. } = run_bp(opts).await;

    assert_eq!(run_fun!(curl --socks5 $bind_addr $http_addr).unwrap(), http_resp);
    assert_eq!(
        run_fun!(curl --socks5-hostname $bind_addr $http_addr).unwrap(),
        http_resp
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_socks5_udp() {
    let opts = Options {
        enable_udp: true,
        client: true,
        ..Default::default()
    };

    let ServiceContext { bind_addr, .. } = run_bp(opts).await;

    let buf = udp_oneshot(&bind_addr, include_bytes!("fixtures/socks5_dns_query.bin")).await;

    // TODO: improve this assertion
    assert_eq!(buf[0..36], include_bytes!("fixtures/dns_resp.bin")[0..36]);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_http() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options {
        client: true,
        ..Default::default()
    };

    let ServiceContext { bind_addr, .. } = run_bp(opts).await;

    assert_eq!(run_fun!(curl -x $bind_addr $http_addr).unwrap(), http_resp);
}
