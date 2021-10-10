use lazy_static::lazy_static;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::lookup_ip::LookupIp;
use trust_dns_resolver::TokioAsyncResolver;

lazy_static! {
  static ref RESOLVER: TokioAsyncResolver = {
    // let mut opts = ResolverConfig::new();
    // opts.add_name_server(NameServerConfig {
        // socket_addr: SocketAddr::new(*ip, port),
        // protocol,
        // tls_dns_name: Some(tls_dns_name.clone()),
        // trust_nx_responses,
    // });
    TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap()
  };
}

pub async fn lookup(addr: &str) -> ResolveResult<LookupIp> {
    RESOLVER.lookup_ip(addr).await
}
