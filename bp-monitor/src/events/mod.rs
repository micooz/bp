pub trait Event {
  fn name() -> String;
}

mod new_connection;

pub use new_connection::NewIncomingConnectionEvent;
