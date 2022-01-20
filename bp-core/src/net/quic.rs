use anyhow::Result;
use quinn::{ClientConfig, ServerConfig};
use rustls::{Certificate, PrivateKey, RootCertStore};
use tokio::fs;

use crate::global::GLOBAL_DATA;

pub async fn init_quinn_server_config(certificate_path: &str, private_key_path: &str) -> Result<()> {
    let cert = fs::read(certificate_path).await?;
    let cert = Certificate(cert);

    let key = fs::read(private_key_path).await?;
    let key = PrivateKey(key);

    let config = ServerConfig::with_single_cert(vec![cert], key)?;

    GLOBAL_DATA.set_quinn_server_config(config);

    Ok(())
}

pub async fn init_quinn_client_config(certificate_path: &str) -> Result<()> {
    let cert = fs::read(certificate_path).await?;
    let cert = Certificate(cert);

    let mut certs = RootCertStore::empty();
    certs.add(&cert)?;

    let config = ClientConfig::with_root_certificates(certs);

    GLOBAL_DATA.set_quinn_client_config(config);

    Ok(())
}

#[derive(Default)]
pub struct QuinnServerConfig {
    inner: Option<ServerConfig>,
}

impl QuinnServerConfig {
    pub fn new(config: ServerConfig) -> Self {
        Self { inner: Some(config) }
    }
    pub fn inner(&self) -> ServerConfig {
        self.inner.as_ref().unwrap().clone()
    }
}

#[derive(Default)]
pub struct QuinnClientConfig {
    inner: Option<ClientConfig>,
}

impl QuinnClientConfig {
    pub fn new(config: ClientConfig) -> Self {
        Self { inner: Some(config) }
    }
    pub fn inner(&self) -> ClientConfig {
        self.inner.as_ref().unwrap().clone()
    }
}
