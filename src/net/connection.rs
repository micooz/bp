use super::{bound::Bound, context::Context};
use crate::{
    net::ConnectionEvent,
    options::Options,
    protocols::{plain::Plain, socks5::Socks5, transparent::Transparent},
    Result, ServiceType,
};
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;

pub struct Connection {
    inbound: Bound,
    outbound: Bound,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: Options) -> Self {
        let ctx = Arc::new(Mutex::new(Context::new(opts)));
        let inbound = Bound::new(Some(socket), ctx.clone());
        let outbound = Bound::new(None, ctx);

        Connection { inbound, outbound }
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

        self.inbound.start_recv(tx);
        self.outbound.start_recv(tx2);

        while let Some(event) = rx.recv().await {
            log::debug!("recv event {:?}", event);

            match event {
                // pipe data from inbound to outbound
                ConnectionEvent::InboundRecv(data) => {
                    self.outbound.write(data).await?;
                }
                // pipe data from outbound to inbound
                ConnectionEvent::OutboundRecv(data) => {
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

        Ok(())
    }
}
