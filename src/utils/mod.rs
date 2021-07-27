use crate::{TcpStreamReader, TcpStreamWriter};
use std::{
    fmt::{Display, LowerHex},
    sync::Arc,
};
use tokio::{net::TcpStream, sync::Mutex};

pub fn split_tcp_stream(
    stream: TcpStream,
) -> (Arc<Mutex<TcpStreamReader>>, Arc<Mutex<TcpStreamWriter>>) {
    let (read_half, write_half) = tokio::io::split(stream);

    let reader = Arc::new(Mutex::new(read_half));
    let writer = Arc::new(Mutex::new(write_half));

    (reader, writer)
}

pub struct ToHex(pub Vec<u8>);

impl Display for ToHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("0x").unwrap();

        self.0.iter().for_each(|x| {
            LowerHex::fmt(x, f).unwrap();
        });

        Ok(())
    }
}
