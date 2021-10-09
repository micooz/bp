use crate::{
    config,
    event::Event,
    net::inbound::{Inbound, InboundSnapshot},
    net::outbound::{Outbound, OutboundSnapshot},
    net::{socket, Address, ServiceType},
    protocol::{DynProtocol, Erp, Plain, TransportProtocol, Universal},
    Options, Result,
};
use tokio::sync;
use tokio::time;

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

    opts: Options,

    inbound: Inbound,

    outbound: Outbound,

    peer_address: Address,

    proxy_address: Option<Address>,

    closed: bool,

    #[cfg(feature = "monitor")]
    monitor_collect_data: MonitorCollectData,
}

impl Connection {
    pub fn new(id: usize, socket: socket::Socket, opts: Options) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let socket_type = socket.socket_type();

        // create inbound
        let inbound = Inbound::new(socket, opts.clone());

        // create outbound
        let outbound = Outbound::new(peer_address, socket_type, opts.clone());

        Connection {
            id,
            inbound,
            outbound,
            peer_address: peer_address.into(),
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

        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(512);

        // [inbound] resolve proxy address
        let resolved = match self.opts.service_type() {
            ServiceType::Client => {
                let universal = Box::new(Universal::new(Some(self.opts.bind.clone())));

                self.inbound.resolve_addr(universal, tx.clone()).await?
            }
            ServiceType::Server => {
                let trans_proto: DynProtocol = match self.opts.protocol {
                    TransportProtocol::Plain => Box::new(Plain::default()),
                    TransportProtocol::EncryptRandomPadding => {
                        Box::new(Erp::new(self.opts.key.clone().unwrap(), self.opts.service_type()))
                    }
                };

                self.inbound.resolve_addr(trans_proto, tx.clone()).await?
            }
        };

        self.proxy_address = resolved.in_proto.get_proxy_address();
        self.update_snapshot().await;

        // [outbound] apply protocol
        self.outbound.set_is_proxy(resolved.is_proxy);
        self.outbound
            .use_protocol(resolved.out_proto, resolved.in_proto, tx.clone())
            .await?;

        self.update_snapshot().await;

        self.handle_events(rx).await?;

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.inbound.close().await?;
        self.outbound.close().await?;

        self.closed = true;
        self.update_snapshot().await;

        Ok(())
    }

    /// handle events from inbound and outbound
    async fn handle_events(&mut self, mut rx: sync::mpsc::Receiver<Event>) -> Result<()> {
        let peer_address = self.peer_address.clone();

        loop {
            let future = rx.recv();

            // timeout check
            let timeout = time::timeout(time::Duration::from_secs(config::READ_WRITE_TIMEOUT_SECONDS), future).await;

            if timeout.is_err() {
                log::warn!(
                    "[{}] no data read/write for {} seconds",
                    peer_address,
                    config::READ_WRITE_TIMEOUT_SECONDS
                );
                self.close().await?;
                break;
            }

            // message check
            let res = timeout.unwrap();
            if res.is_none() {
                break;
            }

            let event = res.unwrap();

            // event handle
            match event {
                Event::ClientEncodeDone(buf) => {
                    self.outbound.send(buf).await?;
                }
                Event::ServerEncodeDone(buf) => {
                    self.inbound.send(buf).await?;
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
                    self.inbound.send(buf).await?;
                }
                Event::ServerDecodeDone(buf) => {
                    self.outbound.send(buf).await?;
                }
                Event::InboundError(_) => {
                    self.close().await?;
                    break;
                }
                Event::OutboundError(_) => {
                    self.close().await?;
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
    proxy_address: Option<Address>,
    inbound_snapshot: InboundSnapshot,
    outbound_snapshot: OutboundSnapshot,
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
