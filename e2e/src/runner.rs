use std::{net::SocketAddr, str::FromStr, sync::Mutex};

use bp_cli::commands::service;
use bp_core::{Address, ClientOptions, Options, ServerOptions, ServiceInfo, Startup};
use tokio::sync::mpsc;

use crate::http_server::{run_http_mock_server, HttpServerContext};

lazy_static::lazy_static! {
    static ref INCREMENTAL_PORT_NUM :Mutex<u16> = Mutex::new(2080);
}

pub struct TestResponse {
    pub bind_addr: SocketAddr,
    pub http_addr: SocketAddr,
    pub http_resp: String,
}

pub async fn run_all(
    client_opts_patch: ClientOptions,
    server_opts_patch: ServerOptions,
    host: Option<&str>,
) -> TestResponse {
    let HttpServerContext { http_addr, http_resp } = run_http_mock_server();

    let key = Some("key".to_string());

    let server = run_bp_custom(
        Options::Server(ServerOptions {
            key: key.clone(),
            ..server_opts_patch
        }),
        host,
        None,
    )
    .await;

    let server_bind = format!("{}:{}", host.unwrap_or(&server.bind_ip.to_string()), server.bind_port);

    let client = run_bp_custom(
        Options::Client(ClientOptions {
            key,
            server_bind: Some(server_bind.parse().unwrap()),
            ..client_opts_patch
        }),
        host,
        None,
    )
    .await;

    TestResponse {
        bind_addr: client.bind_addr,
        http_addr,
        http_resp: http_resp.into(),
    }
}

pub async fn run_bp(mut opts: Options) -> ServiceInfo {
    let host = "127.0.0.1";
    let port = get_auto_port();

    opts.set_bind(Address::from_str(&format!("{}:{}", host, port)).unwrap());
    run_single(opts).await
}

pub async fn run_bp_custom(mut opts: Options, host: Option<&str>, port: Option<u16>) -> ServiceInfo {
    let host = host.unwrap_or("127.0.0.1");
    let port = port.unwrap_or_else(get_auto_port);

    opts.set_bind(Address::from_str(&format!("{}:{}", host, port)).unwrap());
    run_single(opts).await
}

fn get_auto_port() -> u16 {
    let mut port = INCREMENTAL_PORT_NUM.lock().unwrap();
    *port += 1;
    *port
}

async fn run_single(opts: Options) -> ServiceInfo {
    let shutdown = tokio::signal::ctrl_c();
    let (startup_sender, mut startup_receiver) = mpsc::channel::<Startup>(1);

    tokio::spawn(async move {
        let _ = service::run(opts, startup_sender, shutdown).await;
    });

    let startup = startup_receiver.recv().await.unwrap();

    startup.first().unwrap()
}
