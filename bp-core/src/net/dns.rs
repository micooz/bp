use std::net::SocketAddr;

use anyhow::Result;
use tokio::time::{timeout, Duration};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

use crate::{constants, global, Address};

pub async fn init_dns_resolver(dns_server: SocketAddr) -> Result<()> {
    let mut resolver = ResolverConfig::new();

    resolver.add_name_server(NameServerConfig {
        socket_addr: dns_server,
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_nx_responses: true,
    });

    let dns_resolver = TokioAsyncResolver::tokio(resolver, ResolverOpts::default())?;

    global::set_dns_resolver(dns_resolver).await;

    Ok(())
}

pub async fn dns_resolve(addr: &Address) -> Result<SocketAddr> {
    if addr.is_ip() {
        return Ok(addr.as_socket_addr());
    }

    if addr.is_hostname() {
        let name = addr.host();

        let resolver = global::get_dns_resolver();
        let resolver = resolver.lock().await;
        let resolver = resolver.as_ref().unwrap();

        // set a timeout
        let response = timeout(
            Duration::from_secs(constants::DNS_RESOLVE_TIMEOUT_SECONDS),
            resolver.lookup_ip(name.as_str()),
        )
        .await??;

        let ip_list = response
            .iter()
            .map(|ip| SocketAddr::new(ip, addr.port()))
            .collect::<Vec<SocketAddr>>();

        return Ok(ip_list[0]);
    }

    unreachable!()
}
