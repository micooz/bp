use crate::Error;
use bytes::Bytes;
use tokio::sync::mpsc::Sender;

pub type EventSender = Sender<Event>;

#[derive(Debug)]
pub enum Event {
    ClientEncodeDone(Bytes),
    ServerEncodeDone(Bytes),
    ClientDecodeDone(Bytes),
    ServerDecodeDone(Bytes),
    InboundError(Error),
    OutboundError(Error),
}
