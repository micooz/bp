use crate::{event::Event, net, net::inbound, net::outbound, net::socket, protocol, Result};
use std::sync::Arc;
use tokio::sync;

#[cfg(feature = "monitor")]
use bytes::BytesMut;

#[cfg(feature = "monitor")]
const MAX_CACHE_SIZE: usize = 1024;

#[cfg(feature = "monitor")]
struct MonitorCollectData {
    last_decoded_data: BytesMut,
}

pub struct Connection {
    #[allow(dead_code)]
    id: usize,

    opts: net::ConnOptions,

    inbound: Arc<sync::RwLock<inbound::Inbound>>,

    outbound: Arc<sync::RwLock<outbound::Outbound>>,

    proxy_address: Option<net::Address>,

    closed: bool,

    #[cfg(feature = "monitor")]
    monitor_collect_data: MonitorCollectData,
}

impl Connection {
    pub fn new(socket: socket::Socket, opts: net::ConnOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let socket_type = socket.socket_type();

        // create inbound
        let inbound = inbound::Inbound::new(socket, opts.clone());

        // create outbound
        let outbound = outbound::Outbound::new(peer_address, socket_type, opts.clone());

        Connection {
            id: opts.id,
            inbound: Arc::new(sync::RwLock::new(inbound)),
            outbound: Arc::new(sync::RwLock::new(outbound)),
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

        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

        // [inbound] resolve proxy address
        let resolved = match self.opts.service_type {
            net::ServiceType::Client => {
                let universal = Box::new(protocol::Universal::new(Some(self.opts.local_addr.clone())));

                sync::RwLock::write(&self.inbound)
                    .await
                    .resolve_addr(universal, tx.clone())
                    .await?
            }
            net::ServiceType::Server => {
                use protocol::*;

                let trans_proto: DynProtocol = match self.opts.protocol {
                    TransportProtocol::Plain => Box::new(Plain::default()),
                    TransportProtocol::EncryptRandomPadding => {
                        Box::new(Erp::new(self.opts.key.clone().unwrap(), self.opts.service_type))
                    }
                };

                sync::RwLock::write(&self.inbound)
                    .await
                    .resolve_addr(trans_proto, tx.clone())
                    .await?
            }
        };

        self.proxy_address = resolved.in_proto.get_proxy_address();
        self.update_snapshot().await;

        // [outbound] apply protocol
        let mut outbound = sync::RwLock::write(&self.outbound).await;
        outbound.set_is_proxy(resolved.is_proxy);
        outbound
            .use_protocol(resolved.out_proto, resolved.in_proto, tx.clone())
            .await?;

        drop(outbound);

        self.update_snapshot().await;

        self.handle_events(rx).await?;

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    pub async fn force_close(&mut self) -> Result<()> {
        sync::RwLock::write(&self.inbound).await.close().await?;
        sync::RwLock::write(&self.outbound).await.close().await?;

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    /// handle events from inbound and outbound
    async fn handle_events(&self, mut rx: sync::mpsc::Receiver<Event>) -> Result<()> {
        while let Some(event) = rx.recv().await {
            match event {
                Event::ClientEncodeDone(buf) => {
                    sync::RwLock::write(&self.outbound).await.send(buf).await?;
                }
                Event::ServerEncodeDone(buf) => {
                    sync::RwLock::write(&self.inbound).await.send(buf).await?;
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
                    sync::RwLock::write(&self.inbound).await.send(buf).await?;
                }
                Event::ServerDecodeDone(buf) => {
                    sync::RwLock::write(&self.outbound).await.send(buf).await?;
                }
                Event::InboundError(_) => {
                    // close self
                    sync::RwLock::write(&self.inbound).await.close().await?;
                    // close outbound as well
                    sync::RwLock::write(&self.outbound).await.close().await?;

                    break;
                }
                Event::OutboundError(_) => {
                    // close self
                    sync::RwLock::write(&self.outbound).await.close().await?;
                    // close inbound as well
                    sync::RwLock::write(&self.inbound).await.close().await?;

                    break;
                }
            }
        }

        Ok(())
    }

    async fn update_snapshot(&self) {
        #[cfg(feature = "monitor")]
        {
            let mut shared_data = self.opts.shared_data.write().await;
            let snapshot = ConnectionSnapshot {
                id: self.id,
                closed: self.closed,
                proxy_address: self.proxy_address.clone(),
                inbound_snapshot: self.inbound.read().await.snapshot(),
                outbound_snapshot: self.outbound.read().await.snapshot(),
            };
            shared_data.connection_snapshots.insert(self.id, snapshot);
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
