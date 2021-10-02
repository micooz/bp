use crate::{
    config::PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS,
    event::{Event, EventSender},
    net::address::Address,
    net::{
        self,
        io::{TcpStreamReader, TcpStreamWriter},
    },
    protocol::DynProtocol,
    Result, ServiceType,
};
use bytes::Bytes;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::TcpStream,
    sync::Mutex,
    time::{timeout, Duration},
};

pub struct InboundOptions {
    service_type: ServiceType,
}

impl InboundOptions {
    pub fn new(service_type: ServiceType) -> Self {
        Self { service_type }
    }
}

pub struct Inbound {
    opts: InboundOptions,

    /// The read half of current stream
    reader: Arc<Mutex<TcpStreamReader>>,

    /// The write half of current stream
    writer: Arc<Mutex<TcpStreamWriter>>,

    /// The peer address
    peer_address: SocketAddr,

    proxy_address: Option<Address>,

    local_addr: SocketAddr,

    protocol_name: Option<String>,

    /// Whether the bound is closed
    is_closed: bool,
}

impl Inbound {
    pub fn new(socket: TcpStream, opts: InboundOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let local_addr = socket.local_addr().unwrap();

        #[cfg(not(target_os = "linux"))]
        let proxy_address: Option<Address> = None;

        // TODO: get_original_destination_addr always return a value on linux
        #[cfg(target_os = "linux")]
        let proxy_address = {
            use crate::net::linux::get_original_destination_addr;

            match get_original_destination_addr(&socket) {
                Ok(addr) => {
                    log::debug!("{:?}", addr);
                    Some(addr.into())
                }
                Err(_) => None,
            }
        };

        let split = net::io::split_tcp_stream(socket);

        Self {
            opts,
            reader: split.0,
            writer: split.1,
            peer_address,
            proxy_address,
            local_addr,
            protocol_name: None,
            is_closed: false,
        }
    }

    // parse incoming data to get proxy address
    pub async fn resolve_proxy_address(
        &mut self,
        mut in_proto: DynProtocol,
        mut out_proto: DynProtocol,
        tx: EventSender,
    ) -> Result<(DynProtocol, DynProtocol)> {
        self.protocol_name = Some(in_proto.get_name());

        log::info!(
            "[{}] use [{}] protocol",
            self.peer_address,
            self.protocol_name.as_ref().unwrap()
        );

        let (proxy_address, pending_buf) = match self.proxy_address.as_ref() {
            Some(addr) => (addr.clone(), None),
            None => {
                let mut reader = self.reader.lock().await;
                let mut writer = self.writer.lock().await;

                timeout(
                    Duration::from_secs(PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS),
                    in_proto.resolve_proxy_address(&mut reader, &mut writer),
                )
                .await?
                .map_err(|err| format!("resolve proxy address failed due to {}", err))?
            }
        };

        // update proxy_address
        self.proxy_address = Some(proxy_address.clone());

        in_proto.set_proxy_address(proxy_address.clone());
        out_proto.set_proxy_address(proxy_address);

        log::info!(
            "[{}] [{}] resolved target address {}",
            self.peer_address,
            self.protocol_name.as_ref().unwrap(),
            self.proxy_address.as_ref().unwrap(),
        );

        let ret = (in_proto.clone(), out_proto.clone());

        log::info!(
            "[{}] [{}] start receiving data...",
            self.peer_address,
            self.protocol_name.as_ref().unwrap()
        );

        let reader = self.reader.clone();
        let service_type = self.opts.service_type;

        tokio::spawn(async move {
            let mut reader = reader.lock().await;

            // handle pending_buf
            if let Some(buf) = pending_buf {
                match service_type {
                    ServiceType::Client => {
                        reader.cache(&buf);
                        if let Err(err) = out_proto.client_encode(&mut reader, tx.clone()).await {
                            let _ = tx.send(Event::InboundError(err)).await;
                        }
                    }
                    ServiceType::Server => {
                        let _ = tx.send(Event::DecodeDone(buf)).await;
                    }
                }
            }

            loop {
                let res = match service_type {
                    ServiceType::Client => out_proto.client_encode(&mut reader, tx.clone()).await,
                    ServiceType::Server => in_proto.server_decode(&mut reader, tx.clone()).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::InboundError(err)).await;
                    break;
                }
            }
        });

        Ok(ret)
    }

    /// send data to remote
    pub async fn write(&mut self, buf: Bytes) -> Result<()> {
        let mut writer = self.writer.lock().await;

        writer.write_all(&buf).await?;
        writer.flush().await?;

        Ok(())
    }

    /// close the bound
    pub async fn close(&mut self) -> tokio::io::Result<()> {
        let mut writer = self.writer.lock().await;

        // only close the write half
        writer.shutdown().await?;

        self.is_closed = true;

        log::debug!("[{}] closed", self.peer_address);

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn snapshot(&self) -> InboundSnapshot {
        InboundSnapshot {
            peer_addr: self.peer_address,
            local_addr: self.local_addr,
            protocol_name: self.protocol_name.clone(),
        }
    }
}

#[derive(Debug)]
pub struct InboundSnapshot {
    pub peer_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub protocol_name: Option<String>,
}
