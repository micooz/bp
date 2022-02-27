use anyhow::Result;
use bp_core::{Address, ClientOptions, Options, Startup};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

use crate::{commands::service, options::test::TestOptions};

pub async fn run(opts: TestOptions) {
    if let Some(remote_addr) = &opts.http {
        match http_request_via_client(&opts.config, remote_addr.clone()).await {
            Ok(resp) => println!("{}", resp),
            Err(err) => log::error!("{}", err),
        }
    }
}

pub async fn http_request_via_client(client_config: &str, remote_addr: Address) -> Result<String> {
    let opts = Options::Client(ClientOptions {
        config: Some(client_config.into()),
        ..Default::default()
    });

    let (tx, mut rx) = mpsc::channel::<Startup>(1);

    log::info!("starting bp client at {}", opts.bind());

    tokio::spawn(async move {
        service::run(opts, tx, async {}).await.unwrap();
    });

    let startup = rx.recv().await.unwrap();
    let proxy_addr = startup.info().bind_addr;
    let remote_addr = remote_addr.clone();

    log::info!("making connection to bp client {}", proxy_addr);

    let mut stream = TcpStream::connect(proxy_addr).await?;
    let (mut reader, mut writer) = stream.split();

    let req = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", remote_addr);

    log::info!("sending HTTP request to bp client: {:?}", req);

    writer.write_all(req.as_bytes()).await?;
    writer.flush().await?;

    log::info!("waiting for response...");

    let mut resp = String::with_capacity(4096);
    reader.read_to_string(&mut resp).await?;

    Ok(resp)
}
