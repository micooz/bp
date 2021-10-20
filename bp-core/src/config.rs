pub const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

/// The timeout for resolving destination address
pub const DEST_ADDR_RESOLVE_TIMEOUT_SECONDS: u64 = 10;

/// The timeout for resolving ip address
pub const DNS_RESOLVE_TIMEOUT_SECONDS: u64 = 10;

/// The timeout for tcp connect
pub const TCP_CONNECT_TIMEOUT_SECONDS: u64 = 10;

/// The read or write timeout for each connection
pub const READ_WRITE_TIMEOUT_SECONDS: u64 = 60;

// TODO: reduce buffer memory usage
pub const RECV_BUFFER_SIZE: usize = 1024 * 1024; // 1MB

/// The max transmission unit for udp packet
pub const UDP_MTU: usize = 1500;
