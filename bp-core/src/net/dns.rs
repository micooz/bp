use std::net::SocketAddr;

use anyhow::Result;
use tokio::time::{timeout, Duration};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    TokioAsyncResolver,
};

use super::address::Host;
use crate::{config, global::GLOBAL_DATA, Address};

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

pub async fn dns_resolve(addr: &Address) -> Result<SocketAddr> {
    if addr.is_ip() {
        return Ok(addr.as_socket_addr());
    }

    let ip_list = match &addr.host {
        Host::Name(name) => {
            // get pre-init resolver
            let resolver = GLOBAL_DATA.get_dns_resolver();
            let resolver = resolver.read().await;
            let resolver = resolver.as_ref().unwrap();

            // set a timeout
            let response = timeout(
                Duration::from_secs(config::DNS_RESOLVE_TIMEOUT_SECONDS),
                resolver.lookup_ip(name.as_str()),
            )
            .await??;

            response
                .iter()
                .map(|ip| SocketAddr::new(ip, addr.port))
                .collect::<Vec<SocketAddr>>()
        }
        _ => vec![],
    };

    Ok(ip_list[0])
}
