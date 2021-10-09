type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

mod config;
mod event;
mod options;
mod protocol;

pub mod acl;
pub mod context;
pub mod global;
pub mod net;
pub mod utils;

pub use options::{check_options, Options};
pub use protocol::TransportProtocol;
