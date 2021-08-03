use crate::{
    event::Event,
    net::bound::Bound,
    options::{Options, Protocol, ServiceType},
    protocols::{DynProtocol, Erp, Plain, Socks5, Transparent},
    Result,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub struct Connection {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    inbound: Bound,
    outbound: Bound,
    opts: Options,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: Options) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

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
            Protocol::Plain => self.create_protocol(Plain::new()),
            Protocol::EncryptRandomPadding => self.create_protocol(Erp::new(self.opts.key.clone())),
        };

        // apply protocols for inbound and outbound
        match service_type {
            ServiceType::Client => {
                let socks5 = self.create_protocol(Socks5::new());

                let (in_proto, out_proto) = self
                    .inbound
                    .resolve_proxy_address(socks5, protocol, self.tx.clone())
                    .await?;

                self.outbound.use_protocol(out_proto, in_proto, self.tx.clone()).await?;
            }
            ServiceType::Server => {
                let transparent = self.create_protocol(Transparent::new());

                let (in_proto, out_proto) = self
                    .inbound
                    .resolve_proxy_address(protocol, transparent, self.tx.clone())
                    .await?;

                self.outbound.use_protocol(out_proto, in_proto, self.tx.clone()).await?;
            }
        }

        // TODO: add timeout mechanism for bound recv

        while let Some(event) = self.rx.recv().await {
            match event {
                Event::EncodeDone(buf) => {
                    if self.opts.client {
                        self.outbound.write(buf).await?;
                    } else {
                        self.inbound.write(buf).await?;
                    }
                }
                Event::DecodeDone(buf) => {
                    if self.opts.client {
                        self.inbound.write(buf).await?;
                    } else {
                        self.outbound.write(buf).await?;
                    }
                }
                Event::InboundPendingData(buf) => {
                    self.outbound.write(buf).await?;
                }
                Event::InboundClose => {
                    self.outbound.close().await?;

                    if self.inbound.is_closed() {
                        self.rx.close();
                        break;
                    }
                }
                Event::OutboundClose => {
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

    fn create_protocol<T>(&self, proto: T) -> Box<T> {
        Box::new(proto)
    }
}
