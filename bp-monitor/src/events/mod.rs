pub trait Event {}

mod connection_close;
mod new_connection;

pub use connection_close::ConnectionClose;
pub use new_connection::NewConnection;
