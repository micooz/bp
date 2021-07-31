use crate::{
    net::{bound::Bound, BoundEvent},
    options::{Options, Protocol},
    protocols::{erp::Erp, plain::Plain, socks5::Socks5, transparent::Transparent},
    Proto, Result, ServiceType,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub struct Connection {
    tx: Sender<BoundEvent>,
    rx: Receiver<BoundEvent>,
    inbound: Bound,
    outbound: Bound,
    opts: Options,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: Options) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel::<BoundEvent>(32);

        let inbound = Bound::new(Some(socket), opts.clone(), peer_address);
        let outbound = Bound::new(None, opts.clone(), peer_address);

        Connection {
            tx,
            rx,
            inbound,
            outbound,
            opts,
        }
    }

    pub async fn handle(&mut self, service_type: ServiceType) -> Result<()> {
        // select a protocol for transport layer
        let protocol: Proto = match self
            .opts
            .protocol
            .as_ref()
            .unwrap_or(&Protocol::EncryptRandomPadding)
        {
            Protocol::Plain => Box::new(Plain::new()),
            Protocol::EncryptRandomPadding => Box::new(Erp::new(self.opts.key.clone())),
        };

        // apply protocol for inbound and outbound
        match service_type {
            ServiceType::Client => {
                let socks5 = Box::new(Socks5::new());
                let addr = self
                    .inbound
                    .use_proto_inbound(socks5, self.tx.clone())
                    .await?;

                self.outbound.use_proto_outbound(protocol, addr).await?;
            }
            ServiceType::Server => {
                let transparent = Box::new(Transparent::new());

                let addr = self
                    .inbound
                    .use_proto_inbound(protocol, self.tx.clone())
                    .await?;

                self.outbound.use_proto_outbound(transparent, addr).await?;
            }
        }

        let tx_inbound = self.tx.clone();
        let tx_outbound = self.tx.clone();

        let inbound_recv_handle = self.inbound.start_recv(tx_inbound);
        let outbound_recv_handle = self.outbound.start_recv(tx_outbound);

        // TODO: add timeout mechanism for bound recv

        while let Some(event) = self.rx.recv().await {
            // log::debug!("recv event {:?}", event);

            match event {
                // pipe data from inbound to outbound
                BoundEvent::InboundRecv(data) => {
                    let data = if self.opts.client {
                        self.outbound.pack(data)?
                    } else {
                        self.inbound.unpack(data)?
                    };
                    self.outbound.write(data).await?;
                }
                // pipe data from outbound to inbound
                BoundEvent::OutboundRecv(data) => {
                    let data = if self.opts.client {
                        self.outbound.unpack(data)?
                    } else {
                        self.inbound.pack(data)?
                    };
                    self.inbound.write(data).await?;
                }
                // the following data after address parsing
                BoundEvent::InboundPendingData(buf) => {
                    self.outbound.write(buf).await?;
                }
                // inbound closed cause outbound close
                BoundEvent::InboundClose => {
                    log::debug!("trigger outbound close");
                    self.outbound.close().await?;
                }
                // outbound closed cause inbound close
                BoundEvent::OutboundClose => {
                    log::debug!("trigger inbound close");
                    self.inbound.close().await?;
                }
            }
        }

        inbound_recv_handle.await??;
        outbound_recv_handle.await??;

        Ok(())
    }
}
