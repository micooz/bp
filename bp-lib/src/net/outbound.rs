use crate::{
    config::TCP_CONNECT_TIMEOUT_SECONDS,
    event::{Event, EventSender},
    net::{
        self,
        address::Address,
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

pub struct OutboundOptions {
    service_type: ServiceType,
    server_host: Option<String>,
    server_port: Option<u16>,
}

impl OutboundOptions {
    pub fn new(service_type: ServiceType, server_host: Option<String>, server_port: Option<u16>) -> Self {
        Self {
            service_type,
            server_host,
            server_port,
        }
    }
}

pub struct Outbound {
    opts: OutboundOptions,

    /// The read half of current stream
    reader: Option<Arc<Mutex<TcpStreamReader>>>,

    /// The write half of current stream
    writer: Option<Arc<Mutex<TcpStreamWriter>>>,

    /// The peer address
    peer_address: SocketAddr,

    remote_addr: Option<Address>,

    protocol_name: Option<String>,

    /// Whether the bound is closed
    is_closed: bool,
}

impl Outbound {
    pub fn new(peer_address: SocketAddr, opts: OutboundOptions) -> Self {
        Self {
            opts,
            reader: None,
            writer: None,
            peer_address,
            remote_addr: None,
            protocol_name: None,
            is_closed: false,
        }
    }

    // apply transport protocol then make connection to remote
    pub async fn use_protocol(
        &mut self,
        mut out_proto: DynProtocol,
        mut in_proto: DynProtocol,
        tx: EventSender,
    ) -> Result<()> {
        let out_proto_name = out_proto.get_name();
        self.protocol_name = Some(out_proto_name.clone());

        log::info!("[{}] use [{}] protocol", self.peer_address, out_proto_name);

        let remote_addr = self.get_remote_addr(&in_proto);

        self.remote_addr = Some(remote_addr.clone());

        log::info!("[{}] connecting to {}...", self.peer_address, remote_addr,);

        // dns resolve
        let ip_list = remote_addr.dns_resolve().await;
        log::info!("[{}] resolved {} to {:?}", self.peer_address, remote_addr, ip_list);

        self.connect(ip_list).await.map_err(|err| {
            format!(
                "[{}] connect to {} failed due to {}",
                self.peer_address, remote_addr, err
            )
        })?;

        log::info!("[{}] connected to {}", self.peer_address, remote_addr,);

        log::info!("[{}] [{}] start receiving data...", self.peer_address, out_proto_name);

        let reader = self.reader.as_ref().unwrap().clone();
        let service_type = self.opts.service_type;

        tokio::spawn(async move {
            let mut reader = reader.lock().await;
            loop {
                let res = match service_type {
                    ServiceType::Client => out_proto.client_decode(&mut reader, tx.clone()).await,
                    ServiceType::Server => in_proto.server_encode(&mut reader, tx.clone()).await,
                };

                if let Err(err) = res {
                    let _ = tx.send(Event::OutboundError(err)).await;
                    break;
                }
            }
        });

        Ok(())
    }

    /// send data to remote
    pub async fn write(&mut self, buf: Bytes) -> Result<()> {
        let mut writer = self.writer.as_ref().unwrap().lock().await;

        writer.write_all(&buf).await?;
        writer.flush().await?;

        Ok(())
    }

    /// close the bound
    pub async fn close(&mut self) -> tokio::io::Result<()> {
        if let Some(writer) = &self.writer {
            let mut writer = writer.lock().await;

            // only close the write half
            writer.shutdown().await?;

            self.is_closed = true;

            log::debug!("[{}] closed", self.peer_address);
        }

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    pub fn snapshot(&self) -> OutboundSnapshot {
        OutboundSnapshot {
            remote_addr: self.remote_addr.clone(),
            protocol_name: self.protocol_name.clone(),
        }
    }

    fn get_remote_addr(&self, in_proto: &DynProtocol) -> Address {
        if self.opts.service_type.is_server() || self.opts.server_host.is_none() || self.opts.server_port.is_none() {
            in_proto.get_proxy_address().unwrap()
        } else {
            format!(
                "{}:{}",
                self.opts.server_host.as_ref().unwrap(),
                self.opts.server_port.unwrap()
            )
            .parse()
            .unwrap()
        }
    }

    // connect to addr then create self.reader/self.writer
    async fn connect(&mut self, addrs: Vec<SocketAddr>) -> Result<()> {
        let future = TcpStream::connect(addrs.as_slice());
        let socket = timeout(Duration::from_secs(TCP_CONNECT_TIMEOUT_SECONDS), future).await??;

        let split = net::io::split_tcp_stream(socket);

        self.reader = Some(split.0);
        self.writer = Some(split.1);

        Ok(())
    }
}

#[derive(Debug)]
pub struct OutboundSnapshot {
    pub remote_addr: Option<Address>,
    pub protocol_name: Option<String>,
}
