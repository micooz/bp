use std::net::SocketAddr;

use anyhow::Result;
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

use crate::global::GLOBAL_DATA;

pub async fn init_dns_resolver(dns_server: SocketAddr) -> Result<()> {
    let mut resolver = ResolverConfig::new();

    resolver.add_name_server(NameServerConfig {
        socket_addr: dns_server,
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_nx_responses: true,
    });

    let dns_resolver = TokioAsyncResolver::tokio(resolver, ResolverOpts::default())?;

    GLOBAL_DATA.set_dns_resolver(dns_resolver).await;

    Ok(())
}
