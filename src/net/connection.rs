use crate::{
    event::Event,
    net::bound::Bound,
    options::{Options, ServiceType},
    protocol::{Socks5, Transparent},
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
        let protocol = if self.opts.is_transparent_proxy() {
            Box::new(Transparent::new())
        } else {
            self.opts.init_protocol()?
        };

        // [inbound] resolve proxy address
        let (in_proto, out_proto) = match service_type {
            ServiceType::Client => {
                let socks5 = Box::new(Socks5::new());
                self.inbound
                    .resolve_proxy_address(socks5, protocol, self.tx.clone())
                    .await?
            }
            ServiceType::Server => {
                let transparent = Box::new(Transparent::new());
                self.inbound
                    .resolve_proxy_address(protocol, transparent, self.tx.clone())
                    .await?
            }
        };

        // [outbound] apply protocol
        self.outbound.use_protocol(out_proto, in_proto, self.tx.clone()).await?;

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
}
