use bp_core::{Options, StartupInfo};
use e2e::{
    http_server::{run_http_mock_server, HttpServerContext},
    oneshot::tcp_oneshot,
    run_bp::run_bp,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_force_dest() {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let opts = Options {
        client: true,
        force_dest_addr: Some(http_addr.into()),
        ..Options::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = tcp_oneshot(&bind_addr, include_bytes!("fixtures/http_req.bin")).await;
    let resp = String::from_utf8(buf).unwrap();

    assert!(resp.contains(http_resp));
}
