#![feature(slice_group_by)]
#![feature(fs_try_exists)]

mod constants;
mod event;
mod global;
mod io;
mod net;
mod options;
mod protos;
mod services;
mod shutdown;

pub mod acl;
pub mod logging;
pub mod utils;

pub use global::{monitor_log, set_monitor};
pub use net::{
    address::Address,
    connection::Connection,
    dns::init_dns_resolver,
    quic::{init_quic_endpoint_pool, init_quinn_client_config, init_quinn_server_config},
    socket::Socket,
    tls::{init_tls_client_config, init_tls_server_config},
};
pub use options::{
    client::ClientOptions,
    common::{Options, ServiceType},
    server::ServerOptions,
    utils::options_from_file,
};
pub use protos::{EncryptionMethod, HttpBasicAuth};
pub use services::{
    monitor::start_monitor_service, pac::start_pac_service, quic::start_quic_service, tcp::start_tcp_service,
    tls::start_tls_service, udp::start_udp_service, ServiceInfo, ServiceProtocol, Startup,
};
pub use shutdown::Shutdown;
