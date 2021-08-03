use crate::{
    event::{Event, EventSender},
    net::{NetAddr, TcpStreamReader, TcpStreamWriter},
    protocols::DynProtocol,
    utils, Options, Result,
};
use bytes::Bytes;
use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

pub struct Bound {
    /// The options
    opts: Options,

    /// The bound type
    bound_type: BoundType,

    /// The read half of current stream
    reader: Option<Arc<Mutex<TcpStreamReader>>>,

    /// The write half of current stream
    writer: Option<Arc<Mutex<TcpStreamWriter>>>,

    /// The peer address
    peer_address: SocketAddr,

    /// Whether the bound is closed
    is_closed: bool,
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
            peer_address,
            is_closed: false,
        }
    }

    // [inbound] parse incoming data to get proxy address
    pub async fn resolve_proxy_address(
        &mut self,
        mut in_proto: DynProtocol,
        mut out_proto: DynProtocol,
        tx: EventSender,
    ) -> Result<(DynProtocol, DynProtocol)> {
        let in_proto_name = in_proto.get_name();

        log::info!(
            "[{}] [{}] use {} protocol",
            self.bound_type,
            self.peer_address,
            in_proto_name,
        );

        let mut reader = self.reader.as_ref().unwrap().lock().await;
        let mut writer = self.writer.as_ref().unwrap().lock().await;

        let (proxy_address, rest) = in_proto
            .resolve_proxy_address(&mut reader, &mut writer)
            .await
            .map_err(|err| format!("resolve proxy address failed: {}", err))?;

        in_proto.set_proxy_address(proxy_address.clone());
        out_proto.set_proxy_address(proxy_address.clone());

        log::info!(
            "[{}] [{}] [{}] resolved target address {}",
            self.bound_type,
            self.peer_address,
            in_proto_name,
            proxy_address,
        );

        let ret = (in_proto.clone(), out_proto.clone());

        if let Some(buf) = rest {
            tx.send(Event::InboundPendingData(buf)).await?;
        }

        log::info!(
            "[{}] [{}] [{}] start receiving data...",
            self.bound_type,
            self.peer_address,
            in_proto_name,
        );

        let reader = self.reader.as_ref().unwrap().clone();
        let opts = self.opts.clone();

        tokio::spawn(async move {
            let mut reader = reader.lock().await;
            loop {
                let res = if opts.client {
                    out_proto.client_encode(&mut reader, tx.clone()).await
                } else {
                    in_proto.server_decode(&mut reader, tx.clone()).await
                };

                if res.is_err() {
                    tx.send(Event::InboundClose).await.unwrap();
                    break;
                }
            }
        });

        Ok(ret)
    }

    // [outbound] apply protocol and then make connection to remote
    pub async fn use_protocol(
        &mut self,
        mut out_proto: DynProtocol,
        mut in_proto: DynProtocol,
        tx: EventSender,
    ) -> Result<()> {
        let out_proto_name = out_proto.get_name();

        log::info!(
            "[{}] [{}] use {} protocol",
            self.bound_type,
            self.peer_address,
            out_proto_name,
        );

        let remote_addr = if self.opts.client {
            // on client side, make connection to bp server
            self.opts.get_remote_addr()
        } else {
            // on server side, make connection to target host
            in_proto.get_proxy_address().unwrap()
        };

        log::info!(
            "[{}] [{}] [{}] connecting to {}",
            self.bound_type,
            self.peer_address,
            out_proto_name,
            remote_addr,
        );

        self.connect(&remote_addr).await?;

        log::info!(
            "[{}] [{}] [{}] connected to {}",
            self.bound_type,
            self.peer_address,
            out_proto_name,
            remote_addr,
        );

        log::info!(
            "[{}] [{}] [{}] start receiving data...",
            self.bound_type,
            self.peer_address,
            out_proto_name,
        );

        let reader = self.reader.as_ref().unwrap().clone();
        let opts = self.opts.clone();

        tokio::spawn(async move {
            let mut reader = reader.lock().await;
            loop {
                let res = if opts.client {
                    out_proto.client_decode(&mut reader, tx.clone()).await
                } else {
                    in_proto.server_encode(&mut reader, tx.clone()).await
                };

                if res.is_err() {
                    tx.send(Event::OutboundClose).await.unwrap();
                    break;
                }
            }
        });

        Ok(())
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

            // only close the write half
            writer.shutdown().await?;

            self.is_closed = true;

            log::info!("[{}] [{}] closed", self.bound_type, self.peer_address);
        }

        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }

    // connect to addr then create self.reader/self.writer
    async fn connect(&mut self, addr: &NetAddr) -> Result<()> {
        let stream = TcpStream::connect(addr.as_string()).await?;
        let split = utils::net::split_tcp_stream(stream);

        self.reader = Some(split.0);
        self.writer = Some(split.1);

        Ok(())
    }
}

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
