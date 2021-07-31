use crate::{
    net::{address::NetAddr, BoundEvent},
    options::Options,
    utils, Proto, Result, TcpStreamReader, TcpStreamWriter,
};
use bytes::{Bytes, BytesMut};
use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc::Sender,
    sync::Mutex,
    task::JoinHandle,
};

type BoundEventSender = Sender<BoundEvent>;

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
    tx: BoundEventSender,
    reader: Arc<Mutex<TcpStreamReader>>,
    bound_type: BoundType,
}

pub struct Bound {
    opts: Options,

    /// The bound type
    bound_type: BoundType,

    /// The read half of current stream
    reader: Option<Arc<Mutex<TcpStreamReader>>>,

    /// The write half of current stream
    writer: Option<Arc<Mutex<TcpStreamWriter>>>,

    /// The associate protocol
    protocol: Option<Proto>,

    /// The peer address
    peer_address: SocketAddr,
}

impl Bound {
    pub fn new(socket: Option<TcpStream>, opts: Options, peer_address: SocketAddr) -> Bound {
        let mut reader = None;
        let mut writer = None;
        let bound_type;

        if let Some(socket) = socket {
            let split = utils::net::split_tcp_stream(socket);
            reader = Some(split.0);
            writer = Some(split.1);

            bound_type = BoundType::In;
        } else {
            bound_type = BoundType::Out;
        }

        Bound {
            opts,
            bound_type,
            reader,
            writer,
            protocol: None,
            peer_address,
        }
    }

    pub async fn use_proto_inbound(
        &mut self,
        proto: Proto,
        tx: BoundEventSender,
    ) -> Result<NetAddr> {
        log::info!(
            "[{}] [{}] use {} protocol",
            self.bound_type,
            self.peer_address,
            proto.get_name(),
        );

        self.protocol = Some(proto);

        let addr = self.resolve_proxy_address(tx).await?;

        Ok(addr)
    }

    pub async fn use_proto_outbound(&mut self, mut proto: Proto, addr: NetAddr) -> Result<()> {
        log::info!(
            "[{}] [{}] use {} protocol",
            self.bound_type,
            self.peer_address,
            proto.get_name(),
        );

        proto.set_proxy_address(addr.clone());

        self.protocol = Some(proto);

        let remote_addr = {
            if self.opts.client {
                // on client side, make connection to bp server
                self.opts.get_remote_addr()
            } else {
                // on server side, make connection to target host
                addr
            }
        };

        self.connect(&remote_addr).await?;

        Ok(())
    }

    pub fn start_recv(&mut self, tx: BoundEventSender) -> JoinHandle<Result<()>> {
        let reader = self.reader.as_ref().unwrap().clone();
        let bound_type = self.bound_type.clone();

        tokio::spawn(async {
            Self::recv(RecvArgs {
                tx,
                reader,
                bound_type,
            })
            .await
        })
    }

    async fn recv(args: RecvArgs) -> Result<()> {
        let mut reader = args.reader.lock().await;

        loop {
            let mut buffer = BytesMut::with_capacity(4 * 1024);

            if 0 == reader.read_buf(&mut buffer).await? {
                log::debug!("[{}] read_buf return 0", args.bound_type);

                let event = match args.bound_type {
                    BoundType::In => BoundEvent::InboundClose,
                    BoundType::Out => BoundEvent::OutboundClose,
                };

                args.tx.send(event).await?;

                if buffer.is_empty() {
                    return Ok(());
                } else {
                    return Err("connection reset by peer".into());
                }
            }

            let buf = buffer.clone().freeze();

            let event = match args.bound_type {
                BoundType::In => BoundEvent::InboundRecv(buf),
                BoundType::Out => BoundEvent::OutboundRecv(buf),
            };

            args.tx.send(event).await?;
        }
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

            log::debug!("[{}] [{}] close", self.bound_type, self.peer_address);
            // only close the write half
            writer.shutdown().await?;
        }

        Ok(())
    }

    // parse incoming stream to get proxy address
    async fn resolve_proxy_address(&mut self, tx: BoundEventSender) -> Result<NetAddr> {
        let mut reader = self.reader.as_ref().unwrap().lock().await;
        let mut writer = self.writer.as_ref().unwrap().lock().await;

        let proto = self.protocol.as_mut().unwrap();

        let (proxy_address, rest) = proto
            .resolve_proxy_address(&mut reader, &mut writer)
            .await
            .map_err(|err| format!("resolve proxy address failed: {}", err))?;

        log::info!(
            "[{}] [{}] resolved target address {}",
            self.bound_type,
            self.peer_address,
            proxy_address
        );

        if let Some(buf) = rest {
            tx.send(BoundEvent::InboundPendingData(buf)).await?;
        }

        Ok(proxy_address)
    }

    pub fn pack(&mut self, buf: Bytes) -> Result<Bytes> {
        let proto = self.protocol.as_mut().unwrap();

        if self.opts.client {
            proto.client_encode(buf)
        } else {
            proto.server_encode(buf)
        }
    }

    pub fn unpack(&mut self, buf: Bytes) -> Result<Bytes> {
        let proto = self.protocol.as_mut().unwrap();

        if self.opts.client {
            proto.client_decode(buf)
        } else {
            proto.server_decode(buf)
        }
    }

    /// connect to addr
    async fn connect(&mut self, addr: &NetAddr) -> Result<()> {
        let proto_name = self.protocol.as_ref().unwrap().get_name();

        log::info!(
            "[{}] [{}] [{}] connecting to {}",
            self.bound_type,
            self.peer_address,
            proto_name,
            addr
        );

        let stream = TcpStream::connect(addr.as_string()).await?;

        log::info!(
            "[{}] [{}] [{}] connected to {}",
            self.bound_type,
            self.peer_address,
            proto_name,
            addr
        );

        let split = utils::net::split_tcp_stream(stream);

        self.reader = Some(split.0);
        self.writer = Some(split.1);

        Ok(())
    }
}
