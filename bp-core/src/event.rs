use anyhow::Error;
use bytes::Bytes;

#[derive(Debug)]
pub enum Event {
    ClientEncodeDone(Bytes),
    ServerEncodeDone(Bytes),
    ClientDecodeDone(Bytes),
    ServerDecodeDone(Bytes),
    InboundError(Error),
    OutboundError(Error),
}
