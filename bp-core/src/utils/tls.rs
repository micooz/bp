use std::fs;

use anyhow::Result;
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};

pub struct TLS;

impl TLS {
    pub fn generate_cert_and_key(subject_names: Vec<String>, cert_path: &str, key_path: &str) -> Result<()> {
        let cert = generate_simple_self_signed(subject_names).unwrap();
        fs::write(cert_path, cert.serialize_der().unwrap())?;
        fs::write(key_path, cert.serialize_private_key_der())?;
        Ok(())
    }

    pub fn read_cert_from_file(cert_path: &str) -> Result<Certificate> {
        let cert_buf = fs::read(cert_path)?;
        let cert = Certificate(cert_buf);
        Ok(cert)
    }

    pub fn read_key_from_file(key_path: &str) -> Result<PrivateKey> {
        let key_buf = fs::read(key_path)?;
        let key = PrivateKey(key_buf);
        Ok(key)
    }
}
