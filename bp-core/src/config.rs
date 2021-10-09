pub const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

/// The timeout for proxy address resolving
pub const PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS: u64 = 10;

/// The timeout for tcp connect
pub const TCP_CONNECT_TIMEOUT_SECONDS: u64 = 10;

/// The read or write timeout for each connection
pub const READ_WRITE_TIMEOUT_SECONDS: u64 = 60;

// TODO: reduce buffer memory usage
pub const OUTBOUND_RECV_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// The max transmission unit for udp packet
pub const UDP_MTU: usize = 1500;
