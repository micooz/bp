use super::{
    super::{
        options::Options,
        protocols::{plain::Plain, socks5::Socks5, transparent::Transparent},
        Result, ServiceType,
    },
    bound::Bound,
    context::Context,
    ConnectionEvent,
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

pub struct Connection {
    inbound: Bound,
    outbound: Bound,
    opts: Options,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: Options) -> Self {
        let ctx = Arc::new(Mutex::new(Context::new(opts.clone())));
        let inbound = Bound::new(Some(socket), ctx.clone());
        let outbound = Bound::new(None, ctx);

        Connection {
            inbound,
            outbound,
            opts,
        }
    }

    pub async fn handle(&mut self, service_type: ServiceType) -> Result<()> {
        let socks5 = Socks5::new();
        let plain = Plain::new();

        // program <---> socks5|plain <---> plain|transparent <---> target
        match service_type {
            ServiceType::Client => {
                self.inbound.use_proto(socks5).await?;
                self.outbound.use_proto(plain).await?;
            }
            ServiceType::Server => {
                let transparent = Transparent::new();

                self.inbound.use_proto(plain).await?;
                self.outbound.use_proto(transparent).await?;
            }
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel::<ConnectionEvent>(32);
        let tx2 = tx.clone();

        let inbound_recv_handle = self.inbound.start_recv(tx);
        let outbound_recv_handle = self.outbound.start_recv(tx2);

        // TODO: add timeout mechanism for bound recv

        while let Some(event) = rx.recv().await {
            // log::debug!("recv event {:?}", event);

            match event {
                // pipe data from inbound to outbound
                ConnectionEvent::InboundRecv(data) => {
                    let data = if self.opts.client {
                        self.inbound.pack(data)?
                    } else {
                        self.inbound.unpack(data)?
                    };
                    self.outbound.write(data).await?;
                }
                // pipe data from outbound to inbound
                ConnectionEvent::OutboundRecv(data) => {
                    let data = if self.opts.client {
                        self.outbound.unpack(data)?
                    } else {
                        self.outbound.pack(data)?
                    };
                    self.inbound.write(data).await?;
                }
                // inbound closed cause outbound close
                ConnectionEvent::InboundClose => {
                    log::debug!("trigger outbound close");
                    self.outbound.close().await?;
                }
                // outbound closed cause inbound close
                ConnectionEvent::OutboundClose => {
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
