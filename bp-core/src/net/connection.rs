use anyhow::Result;
#[cfg(feature = "monitor")]
use bytes::BytesMut;
use tokio::{sync, time};

use crate::{
    config,
    event::Event,
    global,
    net::{
        address::Address,
        inbound::{Inbound, InboundResolveResult, InboundSnapshot},
        outbound::{Outbound, OutboundSnapshot},
        socket::{Socket, SocketType},
    },
    options::Options,
    protocol::{init_transport_protocol, Direct, Dns, DynProtocol, ProtocolType, ResolvedResult},
};

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

    peer_addr: Address,

    dest_addr: Option<Address>,

    closed: bool,

    #[cfg(feature = "monitor")]
    monitor_collect_data: MonitorCollectData,
}

impl Connection {
    pub fn new(id: usize, socket: Socket, opts: Options) -> Self {
        let peer_addr = socket.peer_addr().unwrap();
        // create inbound
        let inbound = Inbound::new(socket, opts.clone());
        // create outbound
        let outbound = Outbound::new(peer_addr, opts.clone());

        Connection {
            id,
            inbound,
            outbound,
            peer_addr: peer_addr.into(),
            dest_addr: None,
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
        // NOTE: higher buffer size leads to higher memory & cpu usage
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

        let InboundResolveResult { proto: in_proto } = self.inbound.resolve().await?;

        self.inbound.set_protocol_name(in_proto.get_name());

        let resolved = in_proto.get_resolved_result().unwrap();

        // we must drop connection to bp itself, because:
        // connect to bp itself will cause listener.accept() run into infinite loop and
        // produce "No file descriptors available" errors.
        // TODO: how about comparing between localhost and 127.0.0.1?
        if resolved.address == self.opts.bind {
            log::error!(
                "[{}] [{}] detected dest address is bp itself, dropped",
                self.peer_addr,
                self.inbound.socket_type()
            );
            return Ok(());
        }

        self.dest_addr = Some(resolved.address.clone());

        // set outbound socket type
        let inbound_socket_type = self.inbound.socket_type();

        // default the same as inbound
        self.outbound.set_socket_type(inbound_socket_type);

        // client side enable --udp-over-tcp, outbound should be TCP
        if self.opts.udp_over_tcp && matches!(inbound_socket_type, SocketType::Udp) {
            self.outbound.set_socket_type(SocketType::Tcp);
        }

        // server side resolved DNS protocol, outbound should be UDP
        if self.opts.server && matches!(resolved.protocol, ProtocolType::Dns) {
            self.outbound.set_socket_type(SocketType::Udp);
        }

        // check proxy rules then create outbound protocol
        let mut out_proto = if self.check_proxy_rules() {
            self.create_outbound_protocol(resolved).await
        } else {
            self.outbound.set_allow_proxy(false);
            Box::new(Direct::default())
        };

        // sync resolve result to outbound protocol
        out_proto.set_resolved_result(resolved.clone());

        // handle pending_buf at inbound
        if let Some(buf) = resolved.pending_buf.as_ref() {
            self.inbound
                .handle_pending_data(buf.clone(), &mut out_proto, tx.clone())
                .await?;
        }

        // start receiving data from inbound
        if matches!(self.inbound.socket_type(), SocketType::Tcp) {
            self.inbound
                .handle_incoming_data(in_proto.clone(), out_proto.clone(), tx.clone());
        }

        // connect to remote from outbound
        self.outbound.start_connect(&out_proto.get_name(), resolved).await?;

        // start receiving data from outbound
        self.outbound.handle_incoming_data(in_proto, out_proto, tx);

        self.handle_events(rx).await?;

        self.closed = true;

        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.inbound.close().await?;
        self.outbound.close().await?;

        self.closed = true;

        Ok(())
    }

    fn check_proxy_rules(&self) -> bool {
        let dest_addr = self.dest_addr.as_ref().unwrap();
        let dest_addr_host = dest_addr.host.to_string();

        // white list
        if self.opts.client && self.opts.proxy_white_list.is_some() {
            let acl = global::SHARED_DATA.get_acl();

            if !acl.is_host_hit(&dest_addr_host) {
                log::warn!(
                    "[{}] [{}] [{}] is not matched in white list",
                    self.peer_addr,
                    self.inbound.socket_type(),
                    dest_addr_host,
                );
                return false;
            }

            log::info!(
                "[{}] [{}] [{}] is matched in white list",
                self.peer_addr,
                self.inbound.socket_type(),
                dest_addr_host,
            );
        }

        true
    }

    async fn create_outbound_protocol(&self, resolved: &ResolvedResult) -> DynProtocol {
        // bp client should always use bp transport connect to bp server
        if self.opts.client && self.opts.server_bind.is_some() {
            return init_transport_protocol(&self.opts);
        }

        // server dns outbound
        if self.opts.server && matches!(resolved.protocol, ProtocolType::Dns) {
            return Box::new(Dns::new(self.opts.get_dns_server()));
        }

        Box::new(Direct::default())
    }

    /// handle events from inbound and outbound
    async fn handle_events(&mut self, mut rx: sync::mpsc::Receiver<Event>) -> Result<()> {
        let peer_addr = self.peer_addr.clone();
        let socket_type = self.inbound.socket_type();

        loop {
            let future = rx.recv();

            // timeout check
            let timeout = time::timeout(time::Duration::from_secs(config::READ_WRITE_TIMEOUT_SECONDS), future).await;

            if timeout.is_err() {
                log::warn!(
                    "[{}] [{}] no data read/write for {} seconds",
                    peer_addr,
                    socket_type,
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
}

#[derive(Debug)]
pub struct ConnectionSnapshot {
    id: usize,
    closed: bool,
    dest_addr: Option<Address>,
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

        let dest_addr = match self.dest_addr.as_ref() {
            Some(addr) => addr.as_string(),
            None => "<?>".into(),
        };

        format!(
            "{} <--[{} => {}]--> {} <--[{}]--> {} {}",
            peer_addr,
            in_proto_name,
            dest_addr,
            local_addr,
            out_proto_name,
            remote_addr,
            if self.closed { "[closed]" } else { "[alive]" }
        )
    }
}
