use crate::{
    event::Event,
    net::{Inbound, InboundOptions, InboundSnapshot, Outbound, OutboundOptions, OutboundSnapshot},
    protocol::{DynProtocol, Erp, Plain, SocksHttp, Transparent},
    Protocol, Result, ServiceType,
};
use tokio::{
    net::TcpStream,
    sync::mpsc::{Receiver, Sender},
};

pub struct ConnectionOptions {
    service_type: ServiceType,
    protocol: Protocol,
    key: String,
    server_host: Option<String>,
    server_port: Option<u16>,
}

impl ConnectionOptions {
    pub fn new(
        service_type: ServiceType,
        protocol: Protocol,
        key: String,
        server_host: Option<String>,
        server_port: Option<u16>,
    ) -> Self {
        Self {
            service_type,
            protocol,
            key,
            server_host,
            server_port,
        }
    }
}

pub struct Connection {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    inbound: Inbound,
    outbound: Outbound,
    opts: ConnectionOptions,
}

impl Connection {
    pub fn new(socket: TcpStream, opts: ConnectionOptions) -> Self {
        let peer_address = socket.peer_addr().unwrap();
        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

        // create inbound
        let inbound = Inbound::new(socket, InboundOptions::new(opts.service_type));

        // create outbound
        let outbound = Outbound::new(
            peer_address,
            OutboundOptions::new(opts.service_type, opts.server_host.clone(), opts.server_port),
        );

        Connection {
            tx,
            rx,
            inbound,
            outbound,
            opts,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        let protocol = if self.is_transparent_proxy() {
            Box::new(Transparent::new())
        } else {
            self.init_protocol()
        };

        // [inbound] resolve proxy address
        let (in_proto, out_proto) = match self.opts.service_type {
            ServiceType::Client => {
                let socks_http = Box::new(SocksHttp::new());
                self.inbound
                    .resolve_proxy_address(socks_http, protocol, self.tx.clone())
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

        while let Some(event) = self.rx.recv().await {
            match event {
                Event::EncodeDone(buf) => match self.opts.service_type {
                    ServiceType::Client => self.outbound.write(buf).await?,
                    ServiceType::Server => self.inbound.write(buf).await?,
                },
                Event::DecodeDone(buf) => match self.opts.service_type {
                    ServiceType::Client => self.inbound.write(buf).await?,
                    ServiceType::Server => self.outbound.write(buf).await?,
                },
                Event::InboundError(_) => {
                    self.outbound.close().await?;
                    if self.inbound.is_closed() {
                        self.rx.close();
                        break;
                    }
                }
                Event::OutboundError(_) => {
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

    pub async fn force_close(&mut self) -> Result<()> {
        self.inbound.close().await?;
        self.outbound.close().await?;
        Ok(())
    }

    pub fn snapshot(&self) -> ConnectionSnapshot {
        ConnectionSnapshot {
            inbound_snapshot: self.inbound.snapshot(),
            outbound_snapshot: self.outbound.snapshot(),
        }
    }

    fn is_transparent_proxy(&self) -> bool {
        match self.opts.service_type {
            ServiceType::Client => self.opts.server_host.is_none() || self.opts.server_port.is_none(),
            _ => false,
        }
    }

    fn init_protocol(&self) -> DynProtocol {
        match self.opts.protocol {
            Protocol::Plain => Box::new(Plain::new()),
            Protocol::EncryptRandomPadding => Box::new(Erp::new(self.opts.key.clone(), self.opts.service_type)),
        }
    }
}

pub struct ConnectionSnapshot {
    inbound_snapshot: InboundSnapshot,
    outbound_snapshot: OutboundSnapshot,
}

impl ConnectionSnapshot {
    pub fn get_abstract(&self) -> String {
        let peer_addr = self.inbound_snapshot.peer_addr;
        let local_addr = self.inbound_snapshot.local_addr;

        let remote_addr = self.outbound_snapshot.remote_addr.as_ref();
        let remote_addr = if remote_addr.is_none() {
            "<none>".into()
        } else {
            remote_addr.unwrap().as_string()
        };

        let in_proto_name = match self.inbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => &"<none>",
        };

        let out_proto_name = match self.outbound_snapshot.protocol_name.as_ref() {
            Some(name) => name.as_str(),
            None => &"<none>",
        };

        format!(
            "{} <--[{}]--> {} <--[{}]--> {}",
            peer_addr, in_proto_name, local_addr, out_proto_name, remote_addr
        )
    }
}
