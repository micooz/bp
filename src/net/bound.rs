use super::{
    super::{protocols::Protocol, utils, Proto, Result, TcpStreamReader, TcpStreamWriter},
    address::NetAddr,
    context::Context,
    ConnectionEvent,
};
use bytes::{Bytes, BytesMut};
use std::{
    fmt::Display,
    net::SocketAddr,
    sync::{Arc, MutexGuard},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Sender,
    sync::Mutex,
    task::JoinHandle,
};

#[derive(Clone)]
pub enum BoundType {
    In,
    Out,
}

impl Display for BoundType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            BoundType::In => "In",
            BoundType::Out => "Out",
        };
        write!(f, "{}", str)
    }
}

struct RecvArgs {
    tx: Sender<ConnectionEvent>,
    reader: Arc<Mutex<TcpStreamReader>>,
    bound_type: BoundType,
}

pub struct Bound {
    /// The bound type
    bound_type: BoundType,

    /// The read half of current stream
    reader: Option<Arc<Mutex<TcpStreamReader>>>,

    /// The write half of current stream
    writer: Option<Arc<Mutex<TcpStreamWriter>>>,

    /// The associate protocol
    protocol: Option<Proto>,

    /// The context from Connection
    ctx: Arc<std::sync::Mutex<Context>>,
}

impl Bound {
    pub fn new(socket: Option<TcpStream>, ctx: Arc<std::sync::Mutex<Context>>) -> Bound {
        let mut reader = None;
        let mut writer = None;
        let bound_type;

        if let Some(socket) = socket {
            // store peer_address
            ctx.lock().unwrap().peer_address = Some(socket.peer_addr().unwrap());

            let split = utils::split_tcp_stream(socket);
            reader = Some(split.0);
            writer = Some(split.1);

            bound_type = BoundType::In;
        } else {
            bound_type = BoundType::Out;
        }

        Bound {
            bound_type,
            reader,
            writer,
            protocol: None,
            ctx,
        }
    }

    pub async fn use_proto<T>(&mut self, proto: T) -> Result<()>
    where
        T: Protocol + Send + Sync + 'static,
    {
        log::info!(
            "[{}] [{}] use {} protocol",
            self.bound_type,
            self.get_peer_addr().unwrap(),
            proto.get_name(),
        );

        // store protocol
        self.protocol = Some(Box::new(proto));

        match self.get_proxy_address() {
            Some(addr) => {
                let remote_addr = {
                    let ctx = self.get_context();
                    let opts = ctx.opts.as_ref().unwrap();

                    if opts.client {
                        // on client side, make connection to bp server
                        opts.get_remote_addr()
                    } else {
                        // on server side, make connection to target host
                        addr
                    }
                };

                self.connect(&remote_addr).await?;
            }
            None => {
                // parse incoming stream to get proxy address
                let proxy_address = self.resolve_addr().await?;
                let peer_addr = self.get_peer_addr().unwrap();

                log::info!(
                    "[{}] [{}] resolved target address {}",
                    self.bound_type,
                    peer_addr,
                    proxy_address
                );

                let mut ctx = self.get_context();
                ctx.proxy_address = Some(proxy_address);
                ctx.peer_address = Some(peer_addr);
            }
        }

        Ok(())
    }

    pub fn start_recv(&mut self, tx: Sender<ConnectionEvent>) -> JoinHandle<Result<()>> {
        let reader = self.reader.as_ref().unwrap().clone();
        let bound_type = self.bound_type.clone();

        tokio::spawn(async {
            recv(RecvArgs {
                tx,
                reader,
                bound_type,
            })
            .await
        })
    }

    /// send data to remote
    pub async fn write(&mut self, buf: Bytes) -> Result<()> {
        let mut writer = self.writer.as_ref().unwrap().lock().await;

        writer.write_all(&buf).await?;
        writer.flush().await?;

        Ok(())
    }

    /// close the bound
    pub async fn close(&mut self) -> tokio::io::Result<()> {
        if let Some(writer) = &self.writer {
            let mut writer = writer.lock().await;
            let peer_addr = self.get_peer_addr().unwrap();

            log::debug!("[{}] [{}] close", self.bound_type, peer_addr);
            // only close the write half
            writer.shutdown().await?;
        }

        Ok(())
    }

    pub fn pack(&self, buf: Bytes) -> Result<Bytes> {
        let proto = self.protocol.as_ref().unwrap();
        proto.pack(buf)
    }

    pub fn unpack(&self, buf: Bytes) -> Result<Bytes> {
        let proto = self.protocol.as_ref().unwrap();
        proto.unpack(buf)
    }

    /// resolve proxy address
    async fn resolve_addr(&mut self) -> Result<NetAddr> {
        let mut reader = self.reader.as_ref().unwrap().lock().await;
        let mut writer = self.writer.as_ref().unwrap().lock().await;

        let proto = self.protocol.as_mut().unwrap();

        let proxy_address = match proto.resolve_proxy_address(&mut reader, &mut writer).await {
            Ok(addr) => addr,
            Err(err) => {
                return Err(format!("resolve proxy address failed: {}", err).into());
            }
        };

        Ok(proxy_address)
    }

    /// connect to addr
    async fn connect(&mut self, addr: &NetAddr) -> Result<()> {
        let peer_addr = self.get_peer_addr().unwrap();
        let proto_name = self.protocol.as_ref().unwrap().get_name();

        log::info!(
            "[{}] [{}] [{}] connecting to {}",
            self.bound_type,
            peer_addr,
            proto_name,
            addr
        );

        let stream = TcpStream::connect(addr.to_string()).await?;

        log::info!(
            "[{}] [{}] [{}] connected to {}",
            self.bound_type,
            peer_addr,
            proto_name,
            addr
        );

        let split = utils::split_tcp_stream(stream);

        self.reader = Some(split.0);
        self.writer = Some(split.1);

        Ok(())
    }

    fn get_peer_addr(&self) -> Option<SocketAddr> {
        self.get_context().peer_address
    }

    fn get_proxy_address(&self) -> Option<NetAddr> {
        self.get_context().proxy_address.clone()
    }

    fn get_context(&self) -> MutexGuard<Context> {
        self.ctx.as_ref().lock().unwrap()
    }
}

async fn recv(args: RecvArgs) -> Result<()> {
    let mut reader = args.reader.lock().await;

    loop {
        let mut buffer = BytesMut::with_capacity(4 * 1024);

        if 0 == reader.read_buf(&mut buffer).await? {
            log::debug!("[{}] read_buf return 0", args.bound_type);

            let event = match args.bound_type {
                BoundType::In => ConnectionEvent::InboundClose,
                BoundType::Out => ConnectionEvent::OutboundClose,
            };

            args.tx.send(event).await?;

            if buffer.is_empty() {
                return Ok(());
            } else {
                return Err("connection reset by peer".into());
            }
        }

        let buf = buffer.freeze();

        let event = match args.bound_type {
            BoundType::In => ConnectionEvent::InboundRecv(buf),
            BoundType::Out => ConnectionEvent::OutboundRecv(buf),
        };

        args.tx.send(event).await?;
    }
}
