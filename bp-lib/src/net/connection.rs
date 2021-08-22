use crate::{
    event::Event,
    net::inbound::{Inbound, InboundOptions, InboundSnapshot},
    net::outbound::{Outbound, OutboundOptions, OutboundSnapshot},
    protocol::{DynProtocol, Erp, Plain, SocksHttp, Transparent},
    Protocol, Result, ServiceType,
};
use bytes::{BufMut, Bytes, BytesMut};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock};

const MAX_CACHE_SIZE: usize = 1024;

pub struct ConnectionOptions {
    pub id: usize,
    pub service_type: ServiceType,
    pub protocol: Protocol,
    pub key: String,
    pub server_host: Option<String>,
    pub server_port: Option<u16>,
}

pub struct Connection {
    #[allow(dead_code)]
    id: usize,
    inbound: Arc<RwLock<Inbound>>,
    outbound: Arc<RwLock<Outbound>>,
    opts: ConnectionOptions,
    last_decoded_data: BytesMut,
    closed: bool,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: ConnectionOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();

        // create inbound
        let inbound = Inbound::new(socket, InboundOptions::new(opts.service_type));

        // create outbound
        let outbound = Outbound::new(
            peer_address,
            OutboundOptions::new(opts.service_type, opts.server_host.clone(), opts.server_port),
        );

        Connection {
            id: opts.id,
            inbound: Arc::new(RwLock::new(inbound)),
            outbound: Arc::new(RwLock::new(outbound)),
            opts,
            last_decoded_data: BytesMut::with_capacity(MAX_CACHE_SIZE),
            closed: false,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(32);

        let protocol = if self.is_transparent_proxy() {
            Box::new(Transparent::new())
        } else {
            self.init_protocol()
        };

        // [inbound] resolve proxy address
        let (in_proto, out_proto) = match self.opts.service_type {
            ServiceType::Client => {
                let socks_http = Box::new(SocksHttp::new());
                self.inbound
                    .write()
                    .await
                    .resolve_proxy_address(socks_http, protocol, tx.clone())
                    .await?
            }
            ServiceType::Server => {
                let transparent = Box::new(Transparent::new());
                self.inbound
                    .write()
                    .await
                    .resolve_proxy_address(protocol, transparent, tx.clone())
                    .await?
            }
        };

        // [outbound] apply protocol
        self.outbound
            .write()
            .await
            .use_protocol(out_proto, in_proto, tx.clone())
            .await?;

        while let Some(event) = rx.recv().await {
            match event {
                Event::EncodeDone(buf) => match self.opts.service_type {
                    ServiceType::Client => self.outbound.write().await.write(buf).await?,
                    ServiceType::Server => self.inbound.write().await.write(buf).await?,
                },
                Event::DecodeDone(buf) => {
                    // store last decoded data, for monitoring
                    self.last_decoded_data.clear();
                    self.last_decoded_data
                        .put(buf.slice(0..std::cmp::min(buf.len(), MAX_CACHE_SIZE)));

                    match self.opts.service_type {
                        ServiceType::Client => self.inbound.write().await.write(buf).await?,
                        ServiceType::Server => self.outbound.write().await.write(buf).await?,
                    }
                }
                Event::InboundError(_) => {
                    self.outbound.write().await.close().await?;
                    if self.inbound.read().await.is_closed() {
                        rx.close();
                        break;
                    }
                }
                Event::OutboundError(_) => {
                    self.inbound.write().await.close().await?;
                    if self.outbound.read().await.is_closed() {
                        rx.close();
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn force_close(&mut self) -> Result<()> {
        self.inbound.write().await.close().await?;
        self.outbound.write().await.close().await?;
        self.closed = true;
        Ok(())
    }

    // pub fn setup_monitor(&mut self, rx: Receiver<MonitorCommand>) {
    // let id = self.id;
    // let closed = self.closed;
    // let last_decoded_data = &self.last_decoded_data;

    // let inbound = self.inbound.clone();
    // let outbound = self.outbound.clone();

    // emitter.on("snapshot", |_| {
    //     let snapshot = async {
    //         ConnectionSnapshot {
    //             // id: self.id,
    //             // closed: self.closed,
    //             inbound_snapshot: inbound.read().await.snapshot(),
    //             outbound_snapshot: outbound.read().await.snapshot(),
    //         }
    //     };
    //     MonitorCommandReturn::Snapshot(Box::new(snapshot))
    // });

    // emitter.on("dump", move |param| {
    //     if let MonitorCommandParam::Dump((n, k)) = param {
    //         if *id != n {
    //             return MonitorCommandReturn::Dump(None);
    //         }
    //         let data = last_decoded_data.clone().freeze();
    //         let len = std::cmp::min(k as usize, last_decoded_data.len());
    //         let data = data.slice(0..len);
    //         return MonitorCommandReturn::Dump(Some(data));
    //     } else {
    //         return MonitorCommandReturn::Dump(None);
    //     }
    // });
    // }

    fn is_transparent_proxy(&self) -> bool {
        match self.opts.service_type {
            ServiceType::Client => self.opts.server_host.is_none() || self.opts.server_port.is_none(),
            _ => false,
        }
    }

    fn init_protocol(&self) -> DynProtocol {
        match self.opts.protocol {
            Protocol::Plain => Box::new(Plain::new()),
            Protocol::EncryptRandomPadding => Box::new(Erp::new(self.opts.key.clone(), self.opts.service_type)),
        }
    }
}

#[derive(Debug)]
pub struct ConnectionSnapshot {
    // id: usize,
    // closed: bool,
    inbound_snapshot: InboundSnapshot,
    outbound_snapshot: OutboundSnapshot,
}

impl ConnectionSnapshot {
    // pub fn id(&self) -> usize {
    //     self.id
    // }

    pub fn get_abstract(&self) -> String {
        let peer_addr = self.inbound_snapshot.peer_addr;
        let local_addr = self.inbound_snapshot.local_addr;

        let remote_addr = match self.outbound_snapshot.remote_addr.as_ref() {
            Some(addr) => addr.as_string(),
            None => "<none>".into(),
        };

        let in_proto_name = match self.inbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => "<none>",
        };

        let out_proto_name = match self.outbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => "<none>",
        };

        format!(
            "{} <--[{}]--> {} <--[{}]--> {} {}",
            peer_addr,
            in_proto_name,
            local_addr,
            out_proto_name,
            remote_addr,
            "" // if self.closed { "[closed]" } else { "" }
        )
    }
}

// pub type EE = EventEmitter<&'static str, MonitorCommandParam, MonitorCommandReturn>;

#[derive(Debug, Clone)]
pub enum MonitorCommand {
    Snapshot,
    Dump,
}

pub enum MonitorCommandParam {
    Dump(usize, u16),
}

pub enum MonitorCommandReturn {
    Snapshot(Box<dyn std::future::Future<Output = ConnectionSnapshot>>),
    Dump(Option<Bytes>),
}
