use anyhow::Result;
use tokio::{sync::mpsc::Receiver, time};

use crate::{
    constants,
    event::Event,
    global,
    net::{
        address::Address,
        inbound::Inbound,
        outbound::Outbound,
        socket::{Socket, SocketType},
    },
    protos::{init_protocol, Direct, Dns, DynProtocol, ProtocolType, ResolvedResult},
    Options,
};

pub struct Connection {
    opts: Options,
    inbound: Inbound,
    outbound: Outbound,
    peer_addr: Address,
    dest_addr: Option<Address>,
}

impl Connection {
    pub fn new(socket: Socket, opts: Options) -> Self {
        let peer_addr = socket.peer_addr();
        let inbound = Inbound::new(socket, opts.clone());
        let outbound = Outbound::new(peer_addr, opts.clone());

        Connection {
            inbound,
            outbound,
            peer_addr: peer_addr.into(),
            dest_addr: None,
            opts,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        // NOTE: higher buffer size leads to higher memory & cpu usage
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

        let in_proto = self.inbound.resolve().await?;
        let resolved = in_proto.get_resolved_result();

        self.inbound.set_protocol_name(in_proto.get_name());

        // check resolved target address
        self.check_resolved_result(resolved).await?;

        self.dest_addr = Some(resolved.address.clone());

        // check proxy rules then create outbound protocol
        let mut out_proto = if self.check_proxy_rules() {
            self.outbound.set_socket_type(self.get_outbound_socket_type(resolved));
            self.create_outbound_protocol(resolved)
        } else {
            self.outbound.set_socket_type(SocketType::Tcp);
            self.outbound.set_allow_proxy(false);
            Box::new(Direct::default())
        };

        self.outbound.set_protocol_name(&out_proto.get_name());

        // sync resolve result to outbound protocol
        out_proto.set_resolved_result(resolved.clone());

        // handle pending_buf from inbound
        if let Some(buf) = resolved.pending_buf.as_ref() {
            self.inbound
                .handle_pending_data(buf.clone(), &mut out_proto, tx.clone())
                .await?;
        }

        // start receiving data from inbound
        match self.inbound.socket_type() {
            SocketType::Tcp | SocketType::Tls | SocketType::Quic => {
                self.inbound
                    .handle_incoming_data(in_proto.clone(), out_proto.clone(), tx.clone());
            }
            SocketType::Udp => (),
        }

        // connect to remote from outbound
        self.outbound.start_connect(resolved).await?;

        // start receiving data from outbound
        self.outbound.handle_incoming_data(in_proto, out_proto, tx);

        self.handle_events(rx).await?;

        Ok(())
    }

    pub async fn close(&mut self) -> Result<()> {
        self.inbound.close().await?;
        self.outbound.close().await?;
        Ok(())
    }

    fn get_outbound_socket_type(&self, resolved: &ResolvedResult) -> SocketType {
        if self.opts.is_server() {
            // server side resolved DNS protocol, outbound should be UDP
            if matches!(resolved.protocol, ProtocolType::Dns) {
                return SocketType::Udp;
            }
        }

        if self.opts.is_client() {
            if matches!(self.inbound.socket_type(), SocketType::Udp) && !self.opts.udp_over_tcp() {
                // inbound is UDP, but not enable --udp-over-tcp, outbound should be UDP as well
                return SocketType::Udp;
            }
            // client side enable --tls, outbound should be TLS
            if self.opts.tls() {
                return SocketType::Tls;
            }
            // client side enable --quic, outbound should be QUIC
            if self.opts.quic() {
                return SocketType::Quic;
            }
        }

        // default is TCP
        SocketType::Tcp
    }

    async fn check_resolved_result(&self, resolved: &ResolvedResult) -> Result<()> {
        // we must drop connection to bp itself, because:
        // connect to bp itself will cause listener.accept() run into infinite loop and
        // produce "No file descriptors available" errors.
        let resolved_addr = resolved.address.resolve().await?;
        let bind_addr = self.opts.bind().resolve().await?;

        if resolved_addr == bind_addr {
            let msg = format!(
                "[{}] [{}] detected dest address is bp itself, dropped",
                self.peer_addr,
                self.inbound.socket_type()
            );
            log::error!("{}", msg);
            return Err(anyhow::Error::msg(msg));
        }

        Ok(())
    }

    fn check_proxy_rules(&self) -> bool {
        let dest_addr = self.dest_addr.as_ref().unwrap();
        let dest_addr_host = dest_addr.host();

        // white list
        if self.opts.is_client() && self.opts.proxy_white_list().is_some() {
            let acl = global::get_acl();

            if !acl.is_host_hit(&dest_addr_host) {
                log::warn!(
                    "[{}] [{}] [{}] is NOT matched in white list",
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

    fn create_outbound_protocol(&self, resolved: &ResolvedResult) -> DynProtocol {
        // bp client should always use bp transport connect to bp server
        if self.opts.is_client() && self.opts.server_bind().is_some() {
            return init_protocol(self.opts.encryption(), self.opts.key(), self.opts.service_type());
        }

        // server dns outbound
        if self.opts.is_server() && matches!(resolved.protocol, ProtocolType::Dns) {
            return Box::new(Dns::new(self.opts.dns_server()));
        }

        Box::new(Direct::default())
    }

    /// handle events from inbound and outbound
    async fn handle_events(&mut self, mut rx: Receiver<Event>) -> Result<()> {
        let peer_addr = self.peer_addr.clone();
        let socket_type = self.inbound.socket_type();

        loop {
            let future = rx.recv();

            // timeout check
            let timeout = time::timeout(time::Duration::from_secs(constants::READ_WRITE_TIMEOUT_SECONDS), future).await;

            if timeout.is_err() {
                log::warn!(
                    "[{}] [{}] no data read/write for {} seconds",
                    peer_addr,
                    socket_type,
                    constants::READ_WRITE_TIMEOUT_SECONDS
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
                Event::InboundError(_) | Event::OutboundError(_) => {
                    self.close().await?;
                    break;
                }
            }
        }

        Ok(())
    }
}
