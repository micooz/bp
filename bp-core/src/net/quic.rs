use anyhow::Result;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::RootCertStore;

use crate::{
    global,
    utils::{crypto::Crypto, tls::TLS},
};

pub fn init_quinn_server_config(cert_path: &str, key_path: &str) -> Result<()> {
    let cert = TLS::read_cert_from_file(cert_path)?;
    let key = TLS::read_key_from_file(key_path)?;

    let config = ServerConfig::with_single_cert(vec![cert], key)?;
    global::set_quinn_server_config(config);

    Ok(())
}

pub fn init_quinn_client_config(cert_path: &str) -> Result<()> {
    let cert = TLS::read_cert_from_file(cert_path)?;

    let mut certs = RootCertStore::empty();
    certs.add(&cert)?;

    let config = ClientConfig::with_root_certificates(certs);
    global::set_quinn_client_config(config);

    Ok(())
}

pub fn init_quic_endpoint_pool(max_concurrency: u16) {
    let mut pool = EndpointPool::default();
    pool.set_size(max_concurrency);

    global::set_quic_endpoint_pool(pool);
}

#[derive(Default)]
pub struct EndpointPool {
    size: u16,
    data: Vec<Endpoint>,
}

impl EndpointPool {
    pub fn set_size(&mut self, size: u16) {
        self.size = size;
    }

    pub fn random_endpoint(&mut self) -> Result<RandomEndpoint> {
        let mut reuse = true;

        if self.data.len() < self.size as usize {
            self.create()?;
            reuse = false;
        }

        let endpoint = Crypto::random_choose(&self.data).unwrap().clone();

        Ok(RandomEndpoint { inner: endpoint, reuse })
    }

    fn create(&mut self) -> Result<()> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        endpoint.set_default_client_config(global::get_quinn_client_config());

        self.data.push(endpoint);

        Ok(())
    }
}

pub struct RandomEndpoint {
    pub inner: Endpoint,
    pub reuse: bool,
}
