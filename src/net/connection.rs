use crate::{
    net::bound::{Bound, BoundEvent},
    options::{Options, Protocol, ServiceType},
    protocols::{DecodeStatus, DynProtocol, Erp, Plain, Socks5, Transparent},
    Result,
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
        let protocol: DynProtocol = match self.opts.protocol.as_ref().unwrap_or(&Protocol::EncryptRandomPadding) {
            Protocol::Plain => Box::new(Plain::new()),
            Protocol::EncryptRandomPadding => Box::new(Erp::new(self.opts.key.clone())),
        };

        // apply protocols for inbound and outbound
        match service_type {
            ServiceType::Client => {
                let socks5 = Box::new(Socks5::new());
                let addr = self.inbound.resolve_proxy_address(socks5, self.tx.clone()).await?;

                self.outbound.use_protocol(protocol, addr).await?;
            }
            ServiceType::Server => {
                let transparent = Box::new(Transparent::new());
                let addr = self.inbound.resolve_proxy_address(protocol, self.tx.clone()).await?;

                self.outbound.use_protocol(transparent, addr).await?;
            }
        }

        self.inbound.start_recv(self.tx.clone());
        self.outbound.start_recv(self.tx.clone());

        // TODO: add timeout mechanism for bound recv

        while let Some(event) = self.rx.recv().await {
            // log::debug!("recv event {:?}", event);

            match event {
                // pipe data from inbound to outbound
                BoundEvent::InboundRecv(data) => {
                    let data = if self.opts.client {
                        self.outbound.encode(data)?
                    } else {
                        match self.inbound.decode(data)? {
                            DecodeStatus::Pending => {
                                continue;
                            }
                            DecodeStatus::Fulfil(buf) => buf,
                        }
                    };
                    self.outbound.write(data).await?;
                }
                // pipe data from outbound to inbound
                BoundEvent::OutboundRecv(data) => {
                    let data = if self.opts.client {
                        match self.outbound.decode(data)? {
                            DecodeStatus::Pending => {
                                continue;
                            }
                            DecodeStatus::Fulfil(buf) => buf,
                        }
                    } else {
                        self.inbound.encode(data)?
                    };
                    self.inbound.write(data).await?;
                }
                // the following data after address parsing
                BoundEvent::InboundPendingData(buf) => {
                    self.outbound.write(buf).await?;
                }
                // inbound closed cause outbound close
                BoundEvent::InboundClose => {
                    self.outbound.close().await?;

                    if self.inbound.is_closed() {
                        self.rx.close();
                        break;
                    }
                }
                // outbound closed cause inbound close
                BoundEvent::OutboundClose => {
                    self.inbound.close().await?;

                    if self.outbound.is_closed() {
                        self.rx.close();
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
