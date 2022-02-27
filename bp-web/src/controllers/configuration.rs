use bp_cli::{commands::generate, options::generate::ConfigType};
use tide::http::mime;
use tokio::fs;

use crate::{constants::DEFAULT_CONFIG_FILE, options::RunType, state::State};

pub struct ConfigurationController;

impl ConfigurationController {
    pub async fn query(_req: tide::Request<State>) -> tide::Result {
        use serde_json::{from_str, json, Error, Value};

        let resp = json!({
            "config": null,
            "metadata": null,
        });

        let file = fs::read_to_string(DEFAULT_CONFIG_FILE).await;
        if file.is_err() {
            return Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build());
        }

        // parse
        let config: Result<Value, Error> = from_str(&file.unwrap());
        if config.is_err() {
            return Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build());
        }

        let resp = json!({
            "config": config.unwrap(),
            // TODO: "metadata": Cli::metadata(),
            "metadata": null,
        });

        Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build())
    }

    pub async fn create(req: tide::Request<State>) -> tide::Result {
        let state = req.state();

        let config_type = match state.opts.run_type() {
            RunType::Client => ConfigType::Client,
            RunType::Server => ConfigType::Server,
        };

        generate::generate_config(DEFAULT_CONFIG_FILE, config_type).await?;

        Self::query(req).await
    }

    pub async fn modify(mut req: tide::Request<State>) -> tide::Result {
        let new_content = req.body_string().await?;
        if new_content.is_empty() {
            return Ok(tide::Response::builder(403).build());
        }

        fs::write(DEFAULT_CONFIG_FILE, new_content).await?;

        Self::query(req).await
    }
}
