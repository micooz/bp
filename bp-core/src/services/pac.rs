use std::net::SocketAddr;

use anyhow::{Error, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::global;

const PAC_PATH: &str = "/proxy.pac";

pub async fn start_pac_service(bind_addr: SocketAddr, proxy_addr: String) -> Result<()> {
    let listener = TcpListener::bind(bind_addr)
        .await
        .map_err(|err| Error::msg(format!("pac service start failed from {} due to: {}", bind_addr, err)))?;

    log::info!(
        "pac service running at http://{}{}, waiting for requests...",
        bind_addr,
        PAC_PATH,
    );

    tokio::spawn(async move {
        loop {
            let accept = listener.accept().await;

            if let Err(err) = accept {
                log::error!("encountered an error: {}", err);
                break;
            }

            let (stream, peer_addr) = accept.unwrap();

            let proxy_addr = proxy_addr.clone();

            tokio::spawn(async move {
                if let Err(err) = handle_pac_request(stream, peer_addr, &proxy_addr).await {
                    log::error!("[{}] fail to process request due to: {:?}", peer_addr, err);
                }
            });
        }
    });

    Ok(())
}

async fn handle_pac_request(mut stream: TcpStream, peer_addr: SocketAddr, proxy_addr: &str) -> Result<()> {
    let mut buf = BytesMut::with_capacity(1024);

    loop {
        stream.read_buf(&mut buf).await?;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        let buf = &buf[0..buf.len()];

        // via https
        if buf[0] == 0x16 {
            return Err(Error::msg("https request is not currently supported"));
        }

        let status = req.parse(buf)?;

        // waiting request frame complete
        if !status.is_complete() {
            log::debug!("[{}] request is not complete", peer_addr);
            continue;
        }

        log::info!("[{}] {:?}", peer_addr, String::from_utf8(buf.into()));

        let path = req.path.unwrap();

        // check request
        if req.method != Some("GET") || !path.starts_with(PAC_PATH) {
            return Err(Error::msg(format!(
                "invalid method = {:?} or path = {}",
                req.method, path
            )));
        }

        // response pac content
        let acl = global::get_acl();
        let acl_content = acl.to_pac(proxy_addr)?;

        let headers = b"HTTP/1.1 200 OK\r\nContent-Type: application/x-ns-proxy-autoconfig\r\n\r\n";
        stream.write(headers).await?;
        stream.write(acl_content.as_bytes()).await?;
        stream.flush().await?;

        break;
    }

    Ok(())
}
