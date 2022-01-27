use std::sync::Arc;

use anyhow::Result;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use quinn::{ClientConfig, ServerConfig};
use tokio::sync::RwLock;
use trust_dns_resolver::TokioAsyncResolver;

use crate::{acl::AccessControlList, net::quic::RandomEndpoint, EndpointPool};

lazy_static! {
    static ref ACL: Arc<AccessControlList> = Default::default();
    static ref DNS_RESOLVER: Arc<RwLock<Option<TokioAsyncResolver>>> = Default::default();
    static ref QUINN_SERVER_CONFIG: Mutex<Option<ServerConfig>> = Default::default();
    static ref QUINN_CLIENT_CONFIG: Mutex<Option<ClientConfig>> = Default::default();
    static ref QUIC_ENDPOINT_POOL: Mutex<EndpointPool> = Default::default();
}

pub fn get_acl() -> Arc<AccessControlList> {
    ACL.clone()
}

pub fn get_dns_resolver() -> Arc<RwLock<Option<TokioAsyncResolver>>> {
    DNS_RESOLVER.clone()
}

pub async fn set_dns_resolver(resolver: TokioAsyncResolver) {
    let mut inner = DNS_RESOLVER.write().await;
    *inner = Some(resolver);
}

pub fn set_quinn_server_config(config: ServerConfig) {
    let mut server_config = QUINN_SERVER_CONFIG.lock();
    *server_config = Some(config);
}

pub fn get_quinn_server_config() -> ServerConfig {
    let server_config = QUINN_SERVER_CONFIG.lock();
    server_config.clone().unwrap()
}

pub fn set_quinn_client_config(config: ClientConfig) {
    let mut inner = QUINN_CLIENT_CONFIG.lock();
    *inner = Some(config);
}

pub fn get_quinn_client_config() -> ClientConfig {
    let client_config = QUINN_CLIENT_CONFIG.lock();
    client_config.clone().unwrap()
}

pub fn set_quic_endpoint_pool(pool: EndpointPool) {
    let mut quic_endpoint_pool = QUIC_ENDPOINT_POOL.lock();
    *quic_endpoint_pool = pool;
}

pub fn get_random_endpoint() -> Result<RandomEndpoint> {
    let mut quic_endpoint_pool = QUIC_ENDPOINT_POOL.lock();
    quic_endpoint_pool.random_endpoint()
}
