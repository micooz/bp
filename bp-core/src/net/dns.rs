use lazy_static::lazy_static;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::error::ResolveResult;
use trust_dns_resolver::lookup_ip::LookupIp;
use trust_dns_resolver::TokioAsyncResolver;

lazy_static! {
  static ref RESOLVER: TokioAsyncResolver = {
    // TODO: use --dns-server instead, fallback to Google 8.8.8.8
    TokioAsyncResolver::tokio(ResolverConfig::google(), ResolverOpts::default()).unwrap()
  };
}

pub async fn lookup(addr: &str) -> ResolveResult<LookupIp> {
    RESOLVER.lookup_ip(addr).await
}
