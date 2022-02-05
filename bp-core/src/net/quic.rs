use anyhow::Result;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::RootCertStore;

use crate::{
    global,
    utils::{crypto::Crypto, tls},
};

pub fn init_quinn_server_config(cert_path: &str, key_path: &str) -> Result<()> {
    let cert = tls::read_cert_from_file(cert_path)?;
    let key = tls::read_key_from_file(key_path)?;

    let config = ServerConfig::with_single_cert(vec![cert], key)?;
    global::set_quinn_server_config(config);

    Ok(())
}

pub fn init_quinn_client_config(cert_path: &str) -> Result<()> {
    let cert = tls::read_cert_from_file(cert_path)?;

    let mut certs = RootCertStore::empty();
    certs.add(&cert)?;

    let config = ClientConfig::with_root_certificates(certs);
    global::set_quinn_client_config(config);

    Ok(())
}

pub fn init_quic_endpoint_pool(max_concurrency: Option<u16>) {
    let mut pool = EndpointPool::default();

    if let Some(cap) = max_concurrency {
        pool.set_capacity(cap);
    }

    global::set_quic_endpoint_pool(pool);
}

#[derive(Default)]
pub struct EndpointPool {
    capacity: Option<u16>,
    data: Vec<Endpoint>,
}

impl EndpointPool {
    pub fn set_capacity(&mut self, capacity: u16) {
        self.capacity = Some(capacity);
    }

    pub fn random_endpoint(&mut self) -> Result<RandomEndpoint> {
        match self.capacity {
            Some(capacity) => {
                let mut reuse = true;

                if self.data.len() < capacity as usize {
                    let new_endpoint = Self::create()?;
                    self.data.push(new_endpoint);
                    reuse = false;
                }

                let endpoint = Crypto::random_choose(&self.data).unwrap().clone();

                Ok(RandomEndpoint { inner: endpoint, reuse })
            }
            None => Ok(RandomEndpoint {
                inner: Self::create()?,
                reuse: false,
            }),
        }
    }

    fn create() -> Result<Endpoint> {
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        endpoint.set_default_client_config(global::get_quinn_client_config());
        Ok(endpoint)
    }
}

pub struct RandomEndpoint {
    pub inner: Endpoint,
    pub reuse: bool,
}
