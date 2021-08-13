use crate::Error;
use bytes::Bytes;
use tokio::sync::mpsc::Sender;

pub type EventSender = Sender<Event>;

#[derive(Debug)]
pub enum Event {
    EncodeDone(Bytes),
    DecodeDone(Bytes),
    InboundError(Error),
    OutboundError(Error),
}
