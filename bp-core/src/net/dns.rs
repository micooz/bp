use crate::{global, Result};
use std::net::SocketAddr;
use trust_dns_resolver::config::Protocol;
use trust_dns_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

pub async fn init_dns_resolver(dns_server: SocketAddr) -> Result<()> {
    let mut resolver = ResolverConfig::new();

    resolver.add_name_server(NameServerConfig {
        socket_addr: dns_server,
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_nx_responses: true,
    });

    let dns_resolver = TokioAsyncResolver::tokio(resolver, ResolverOpts::default())?;

    global::SHARED_DATA.set_dns_resolver(dns_resolver).await;

    Ok(())
}
