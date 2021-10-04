use crate::{
    event, net, net::inbound, net::outbound, net::socket, protocol, Result, ServiceType, SharedData, TransportProtocol,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(feature = "monitor")]
use bytes::BytesMut;

#[cfg(feature = "monitor")]
const MAX_CACHE_SIZE: usize = 1024;

#[cfg(feature = "monitor")]
struct MonitorCollectData {
    last_decoded_data: BytesMut,
}

pub struct ConnectionOptions {
    pub id: usize,
    pub service_type: ServiceType,
    pub protocol: TransportProtocol,
    pub key: Option<String>,
    pub local_addr: net::Address,
    pub server_addr: Option<net::Address>,
    pub shared_data: Arc<RwLock<SharedData>>,
}

pub struct Connection {
    #[allow(dead_code)]
    id: usize,
    inbound: Arc<RwLock<inbound::Inbound>>,
    outbound: Arc<RwLock<outbound::Outbound>>,
    proxy_address: Option<net::Address>,
    opts: ConnectionOptions,
    closed: bool,

    #[cfg(feature = "monitor")]
    monitor_collect_data: MonitorCollectData,
}

impl Connection {
    pub fn new(socket: socket::Socket, opts: ConnectionOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let socket_type = socket.socket_type();

        // create inbound
        let inbound = inbound::Inbound::new(
            socket,
            inbound::InboundOptions {
                service_type: opts.service_type,
            },
        );

        // create outbound
        let outbound = outbound::Outbound::new(
            peer_address,
            outbound::OutboundOptions {
                service_type: opts.service_type,
                server_addr: opts.server_addr.clone(),
                socket_type,
            },
        );

        Connection {
            id: opts.id,
            inbound: Arc::new(RwLock::new(inbound)),
            outbound: Arc::new(RwLock::new(outbound)),
            proxy_address: None,
            opts,
            #[cfg(feature = "monitor")]
            monitor_collect_data: MonitorCollectData {
                last_decoded_data: BytesMut::with_capacity(MAX_CACHE_SIZE),
            },
            closed: false,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub async fn handle(&mut self) -> Result<()> {
        self.update_snapshot().await;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<event::Event>(32);

        let trans_proto = self.create_transport_protocol();

        // [inbound] resolve proxy address
        let (in_proto, out_proto) = match self.opts.service_type {
            ServiceType::Client => {
                let socks_http = Box::new(protocol::SocksHttp::new(Some(self.opts.local_addr.clone())));
                RwLock::write(&self.inbound)
                    .await
                    .resolve_proxy_address(socks_http, trans_proto, tx.clone())
                    .await?
            }
            ServiceType::Server => {
                let direct = Box::new(protocol::Direct::new());
                RwLock::write(&self.inbound)
                    .await
                    .resolve_proxy_address(trans_proto, direct, tx.clone())
                    .await?
            }
        };

        self.proxy_address = in_proto.get_proxy_address();
        self.update_snapshot().await;

        // [outbound] apply protocol
        RwLock::write(&self.outbound)
            .await
            .use_protocol(out_proto, in_proto, tx.clone())
            .await?;

        self.update_snapshot().await;

        while let Some(event) = rx.recv().await {
            use event::Event;

            match event {
                Event::ClientEncodeDone(buf) => {
                    RwLock::write(&self.outbound).await.send(buf).await?;
                }
                Event::ServerEncodeDone(buf) => {
                    RwLock::write(&self.inbound).await.send(buf).await?;
                }
                Event::ClientDecodeDone(buf) => {
                    // TODO: store last decoded data, for monitoring
                    // #[cfg(feature = "monitor")]
                    // {
                    //     self.monitor_collect_data.last_decoded_data.clear();
                    //     self.monitor_collect_data
                    //         .last_decoded_data
                    //         .put(buf.slice(0..std::cmp::min(buf.len(), MAX_CACHE_SIZE)));
                    // }
                    RwLock::write(&self.inbound).await.send(buf).await?;
                }
                Event::ServerDecodeDone(buf) => {
                    RwLock::write(&self.outbound).await.send(buf).await?;
                }
                Event::InboundError(_) => {
                    // close self
                    RwLock::write(&self.inbound).await.close().await?;
                    // close outbound as well
                    RwLock::write(&self.outbound).await.close().await?;

                    break;
                }
                Event::OutboundError(_) => {
                    // close self
                    RwLock::write(&self.outbound).await.close().await?;
                    // close inbound as well
                    RwLock::write(&self.inbound).await.close().await?;

                    break;
                }
            }
        }

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    pub async fn force_close(&mut self) -> Result<()> {
        RwLock::write(&self.inbound).await.close().await?;
        RwLock::write(&self.outbound).await.close().await?;

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    async fn update_snapshot(&self) {
        let mut shared_data = self.opts.shared_data.write().await;
        let snapshot = ConnectionSnapshot {
            id: self.id,
            closed: self.closed,
            proxy_address: self.proxy_address.clone(),
            inbound_snapshot: self.inbound.read().await.snapshot(),
            outbound_snapshot: self.outbound.read().await.snapshot(),
        };
        shared_data.conns.insert(self.id, snapshot);
    }

    fn create_transport_protocol(&self) -> protocol::DynProtocol {
        if self.opts.service_type.is_client() && self.opts.server_addr.is_none() {
            Box::new(protocol::Direct::new())
        } else {
            match self.opts.protocol {
                TransportProtocol::Plain => Box::new(protocol::Plain::new()),
                TransportProtocol::EncryptRandomPadding => Box::new(protocol::Erp::new(
                    self.opts.key.clone().unwrap(),
                    self.opts.service_type,
                )),
            }
        }
    }
}

#[derive(Debug)]
pub struct ConnectionSnapshot {
    id: usize,
    closed: bool,
    proxy_address: Option<net::Address>,
    inbound_snapshot: inbound::InboundSnapshot,
    outbound_snapshot: outbound::OutboundSnapshot,
}

impl ConnectionSnapshot {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn get_abstract(&self) -> String {
        let peer_addr = self.inbound_snapshot.peer_addr;
        let local_addr = self.inbound_snapshot.local_addr;

        let remote_addr = match self.outbound_snapshot.remote_addr.as_ref() {
            Some(addr) => addr.as_string(),
            None => "<?>".into(),
        };

        let in_proto_name = match self.inbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => "<?>",
        };

        let out_proto_name = match self.outbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => "<?>",
        };

        let proxy_address = match self.proxy_address.as_ref() {
            Some(addr) => addr.as_string(),
            None => "<?>".into(),
        };

        format!(
            "{} <--[{} => {}]--> {} <--[{}]--> {} {}",
            peer_addr,
            in_proto_name,
            proxy_address,
            local_addr,
            out_proto_name,
            remote_addr,
            if self.closed { "[closed]" } else { "[alive]" }
        )
    }
}
