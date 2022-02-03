use std::sync::Arc;

use anyhow::Result;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tokio::sync::RwLock;
use trust_dns_resolver::TokioAsyncResolver;

use crate::{
    acl::AccessControlList,
    net::quic::{EndpointPool, RandomEndpoint},
};

lazy_static! {
    static ref ACL: Arc<AccessControlList> = Default::default();
    static ref DNS_RESOLVER: Arc<RwLock<Option<TokioAsyncResolver>>> = Default::default();
    static ref TLS_SERVER_CONFIG: Mutex<Option<rustls::ServerConfig>> = Default::default();
    static ref TLS_CLIENT_CONFIG: Mutex<Option<rustls::ClientConfig>> = Default::default();
    static ref QUINN_SERVER_CONFIG: Mutex<Option<quinn::ServerConfig>> = Default::default();
    static ref QUINN_CLIENT_CONFIG: Mutex<Option<quinn::ClientConfig>> = Default::default();
    static ref QUINN_ENDPOINT_POOL: Mutex<EndpointPool> = Default::default();
}

// acl

pub fn get_acl() -> Arc<AccessControlList> {
    ACL.clone()
}

// dns_resolver

pub fn get_dns_resolver() -> Arc<RwLock<Option<TokioAsyncResolver>>> {
    DNS_RESOLVER.clone()
}

pub async fn set_dns_resolver(resolver: TokioAsyncResolver) {
    let mut inner = DNS_RESOLVER.write().await;
    *inner = Some(resolver);
}

// tls

pub fn set_tls_server_config(config: rustls::ServerConfig) {
    let mut server_config = TLS_SERVER_CONFIG.lock();
    *server_config = Some(config);
}

pub fn get_tls_server_config() -> rustls::ServerConfig {
    let server_config = TLS_SERVER_CONFIG.lock();
    server_config.clone().unwrap()
}

pub fn set_tls_client_config(config: rustls::ClientConfig) {
    let mut inner = TLS_CLIENT_CONFIG.lock();
    *inner = Some(config);
}

pub fn get_tls_client_config() -> rustls::ClientConfig {
    let client_config = TLS_CLIENT_CONFIG.lock();
    client_config.clone().unwrap()
}

// quinn

pub fn set_quinn_server_config(config: quinn::ServerConfig) {
    let mut server_config = QUINN_SERVER_CONFIG.lock();
    *server_config = Some(config);
}

pub fn get_quinn_server_config() -> quinn::ServerConfig {
    let server_config = QUINN_SERVER_CONFIG.lock();
    server_config.clone().unwrap()
}

pub fn set_quinn_client_config(config: quinn::ClientConfig) {
    let mut inner = QUINN_CLIENT_CONFIG.lock();
    *inner = Some(config);
}

pub fn get_quinn_client_config() -> quinn::ClientConfig {
    let client_config = QUINN_CLIENT_CONFIG.lock();
    client_config.clone().unwrap()
}

pub fn set_quic_endpoint_pool(pool: EndpointPool) {
    let mut quic_endpoint_pool = QUINN_ENDPOINT_POOL.lock();
    *quic_endpoint_pool = pool;
}

pub fn get_random_endpoint() -> Result<RandomEndpoint> {
    let mut quic_endpoint_pool = QUINN_ENDPOINT_POOL.lock();
    quic_endpoint_pool.random_endpoint()
}
