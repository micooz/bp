use std::path::Path;

use anyhow::{Error, Result};
use bp_core::{utils::tls, ClientOptions, ConfigType, GenerateOptions, ServerOptions};
use tokio::fs;

use crate::utils::exit::{exit, ExitError};

pub async fn run(opts: GenerateOptions) {
    if let Err(err) = opts.check() {
        log::error!("{}", err);
        exit(ExitError::ArgumentsError);
    }
    if let Err(err) = handle(opts).await {
        log::error!("{}", err);
    }
}

async fn handle(opts: GenerateOptions) -> Result<()> {
    // generate bp configuration
    if let Some(config) = &opts.config {
        let ext = Path::new(config)
            .extension()
            .map(|v| v.to_str().unwrap())
            .unwrap_or("json");

        let mut client_opts = ClientOptions::default();
        let mut server_opts = ServerOptions::default();

        client_opts.bind = "127.0.0.1:1080".parse().unwrap();
        client_opts.key = Some("__some_key__".to_string());
        client_opts.server_bind = Some("__some_where__:3000".parse().unwrap());

        client_opts.bind = "0.0.0.0:3000".parse().unwrap();
        server_opts.key = "__some_key__".to_string();

        let content = match ext {
            "yml" | "yaml" => match opts.config_type {
                ConfigType::Client => serde_yaml::to_string(&client_opts)?,
                ConfigType::Server => serde_yaml::to_string(&server_opts)?,
            },
            "json" => match opts.config_type {
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

        return Ok(());
    }

    // generate tls certificate and private key
    if opts.certificate {
        let hostname = opts.hostname.unwrap();
        tls::generate_cert_and_key(vec![hostname], "cert.der", "key.der")?;
    }

    Ok(())
}
