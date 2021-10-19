type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

mod acl;
mod config;
mod context;
mod event;
mod net;
mod options;
mod protocol;

pub mod global;
pub mod utils;

pub use acl::{AccessControlList, DomainItem, DomainRule};
pub use context::SharedData;
pub use net::address::Address;
pub use net::connection::Connection;
pub use net::service::{start_service, StartupInfo};
pub use net::socket::Socket;
pub use options::{check_options, Options};
pub use protocol::TransportProtocol;
