use anyhow::Result;
use rcgen::generate_simple_self_signed;

pub struct TLS;

impl TLS {
    pub async fn gen_cert_and_key(subject_names: Vec<String>, cert_path: &str, key_path: &str) -> Result<()> {
        let cert = generate_simple_self_signed(subject_names).unwrap();

        tokio::fs::write(cert_path, cert.serialize_der().unwrap()).await?;
        tokio::fs::write(key_path, cert.serialize_private_key_der()).await?;

        Ok(())
    }
}
