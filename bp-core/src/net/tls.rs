use anyhow::Result;
use rustls::{ClientConfig, RootCertStore, ServerConfig};

use crate::{global, utils::tls};

pub fn init_tls_server_config(cert_path: &str, key_path: &str) -> Result<()> {
    let cert = tls::read_cert_from_file(cert_path)?;
    let key = tls::read_key_from_file(key_path)?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    global::set_tls_server_config(config);

    Ok(())
}

pub fn init_tls_client_config(cert_path: &str) -> Result<()> {
    let cert = tls::read_cert_from_file(cert_path)?;

    let mut certs = RootCertStore::empty();
    certs.add(&cert)?;

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(certs)
        .with_no_client_auth();

    global::set_tls_client_config(config);

    Ok(())
}
