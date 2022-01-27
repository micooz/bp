use anyhow::Result;
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::{Certificate, PrivateKey, RootCertStore};
use tokio::fs;

use crate::{global, utils::crypto::Crypto};

pub async fn init_quinn_server_config(certificate_path: &str, private_key_path: &str) -> Result<()> {
    let cert = fs::read(certificate_path).await?;
    let cert = Certificate(cert);

    let key = fs::read(private_key_path).await?;
    let key = PrivateKey(key);

    let config = ServerConfig::with_single_cert(vec![cert], key)?;

    global::set_quinn_server_config(config);

    Ok(())
}

pub async fn init_quinn_client_config(certificate_path: &str) -> Result<()> {
    let cert = fs::read(certificate_path).await?;
    let cert = Certificate(cert);

    let mut certs = RootCertStore::empty();
    certs.add(&cert)?;

    let config = ClientConfig::with_root_certificates(certs);

    global::set_quinn_client_config(config);

    Ok(())
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
