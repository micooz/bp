pub const SERVICE_CONNECTION_THRESHOLD: usize = 1024;

/// The timeout for proxy address resolving
pub const PROXY_ADDRESS_RESOLVE_TIMEOUT_SECONDS: u64 = 10;

/// The timeout for tcp connect
pub const TCP_CONNECT_TIMEOUT_SECONDS: u64 = 10;

/// The timeout for outbound recv
pub const OUTBOUND_RECV_TIMEOUT_SECONDS: u64 = 10;

/// The max transmission unit for udp packet
pub const UDP_MTU: usize = 1500;
