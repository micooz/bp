use std::path::Path;

use anyhow::{Error, Result};
use bp_core::{utils::tls, ClientOptions, ServerOptions};
use tokio::fs;

use crate::{
    options::generate::{ConfigType, GenerateOptions},
    utils::exit::{exit, ExitError},
};

pub async fn run(opts: GenerateOptions) {
    if let Err(err) = opts.check() {
        log::error!("{}", err);
        exit(ExitError::ArgumentsError);
    }

    let mut res = Result::Ok(());

    // generate bp configuration
    if let Some(config) = &opts.config {
        res = generate_config(config, opts.config_type).await;
    }
    // generate tls certificate and private key
    if opts.certificate {
        res = generate_certificate(&opts.hostname.unwrap(), "cert.der", "key.der").await;
    }

    if let Err(err) = res {
        log::error!("{}", err);
    }
}

pub async fn generate_config(config: &str, config_type: ConfigType) -> Result<()> {
    let ext = Path::new(config)
        .extension()
        .map(|v| v.to_str().unwrap())
        .unwrap_or("json");

    let mut client_opts = ClientOptions::default();
    let mut server_opts = ServerOptions::default();

    client_opts.key = Some("__some_key__".to_string());
    client_opts.server_bind = Some("__some_where__:3000".parse().unwrap());

    server_opts.bind = "__some_where__:3000".parse().unwrap();
    server_opts.key = Some("__some_key__".to_string());

    let content = match ext {
        "yml" | "yaml" => match config_type {
            ConfigType::Client => serde_yaml::to_string(&client_opts)?,
            ConfigType::Server => serde_yaml::to_string(&server_opts)?,
        },
        "json" => match config_type {
            ConfigType::Client => serde_json::to_string_pretty(&client_opts)?,
            ConfigType::Server => serde_json::to_string_pretty(&server_opts)?,
        },
        _ => {
            return Err(Error::msg(format!(
                "unknown file extension: {}, only support \"json\", \"yaml\" or \"yml\"",
                ext
            )));
        }
    };

    fs::write(config, content).await?;

    Ok(())
}

pub async fn generate_certificate(hostname: &str, cert_path: &str, key_path: &str) -> Result<()> {
    tls::generate_cert_and_key(vec![hostname.to_string()], cert_path, key_path)?;
    Ok(())
}
