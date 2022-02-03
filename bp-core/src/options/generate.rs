#[derive(clap::Args)]
pub struct GenerateOptions {
    /// Generate self-signed TLS certificates(in DER format) to CWD
    #[clap(long)]
    pub certificate: bool,

    /// Hostname for generating TLS certificates
    #[clap(long)]
    pub hostname: Option<String>,
}
