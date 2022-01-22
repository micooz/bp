use std::net::SocketAddr;

use bp_core::Options;

use crate::{
    http_server::{run_http_mock_server, HttpServerContext},
    run_bp::run_bp,
};

pub struct TestResponse {
    pub bind_addr: String,
    pub http_addr: SocketAddr,
    pub http_resp: String,
}

pub async fn run_all(opts: Options) -> TestResponse {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let key = Some("key".to_string());

    let server = run_bp(Options {
        server: true,
        key: key.clone(),
        proxy_white_list: None,
        ..opts.clone()
    })
    .await;

    let client = run_bp(Options {
        client: true,
        key,
        server_bind: Some(server.bind_addr.clone()),
        privatekey: None,
        ..opts
    })
    .await;

    TestResponse {
        bind_addr: client.bind_addr.as_string(),
        http_addr,
        http_resp: http_resp.into(),
    }
}
