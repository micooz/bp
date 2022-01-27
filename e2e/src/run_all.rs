use std::net::SocketAddr;

use bp_core::Options;

use crate::{
    http_server::{run_http_mock_server, HttpServerContext},
    run_bp::run_bp,
};

pub struct TestResponse {
    pub bind_addr: SocketAddr,
    pub http_addr: SocketAddr,
    pub http_resp: String,
}

pub async fn run_all(partial_opts: Options, host: Option<&str>) -> TestResponse {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server(None);

    let key = Some("key".to_string());

    let server = run_bp(
        Options {
            server: true,
            key: key.clone(),
            proxy_white_list: None,
            ..partial_opts.clone()
        },
        host,
    )
    .await;

    let server_bind = format!("{}:{}", host.unwrap_or(&server.bind_ip.to_string()), server.bind_port);

    let client = run_bp(
        Options {
            client: true,
            key,
            server_bind: Some(server_bind.parse().unwrap()),
            tls_key: None,
            ..partial_opts
        },
        host,
    )
    .await;

    TestResponse {
        bind_addr: client.bind_addr,
        http_addr,
        http_resp: http_resp.into(),
    }
}
