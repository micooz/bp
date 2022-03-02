use std::{fs, sync::Arc};

use bp_cli::{commands::generate, options::generate::ConfigType};
use parking_lot::Mutex;
use serde_json::{from_str, json, Error, Value};
use tide::http::mime;

use crate::{
    constants::{DEFAULT_ACL_FILE, DEFAULT_CERT_FILE, DEFAULT_CONFIG_FILE, DEFAULT_PRIV_KEY_FILE},
    options::RunType,
    state::State,
};

pub struct ConfigController;

impl ConfigController {
    pub async fn query(req: tide::Request<State>) -> tide::Result {
        let state = req.state();

        let resp = json!({
            "config": null,
            "metadata": null,
        });

        let file = tokio::fs::read_to_string(DEFAULT_CONFIG_FILE).await;
        if file.is_err() {
            return Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build());
        }

        // parse
        let config: Result<Value, Error> = from_str(&file.unwrap());
        if config.is_err() {
            return Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build());
        }

        let config = Arc::new(Mutex::new(config.unwrap()));

        // check existence of tls_cert, tls_key and acl
        Self::check_config_field(config.clone(), "tls_cert");
        Self::check_config_field(config.clone(), "acl");

        if state.opts.server {
            Self::check_config_field(config.clone(), "tls_key");
        }

        let resp = json!({
            "config": *config.lock(),
            "metadata": null, // TODO: "metadata": Cli::metadata(),
        });

        Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build())
    }

    pub async fn query_acl(_req: tide::Request<State>) -> tide::Result {
        let file = tokio::fs::read_to_string(DEFAULT_ACL_FILE).await;

        if file.is_err() {
            return Ok(tide::Response::builder(404).build());
        }

        Ok(tide::Response::builder(200)
            .body(file.unwrap())
            .content_type(mime::PLAIN)
            .build())
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

    pub async fn create_tls_config(mut req: tide::Request<State>) -> tide::Result {
        #[derive(serde::Deserialize)]
        struct Params {
            hostname: String,
        }

        let Params { hostname } = req.body_json().await?;

        generate::generate_certificate(&hostname, DEFAULT_CERT_FILE, DEFAULT_PRIV_KEY_FILE).await?;

        Ok(tide::Response::builder(200).build())
    }

    pub async fn modify(mut req: tide::Request<State>) -> tide::Result {
        let new_content = req.body_string().await?;
        if new_content.is_empty() {
            return Ok(tide::Response::builder(403).build());
        }

        tokio::fs::write(DEFAULT_CONFIG_FILE, new_content).await?;

        Ok(tide::Response::builder(200).build())
    }

    fn check_config_field(config: Arc<Mutex<Value>>, field: &'static str) {
        let mut config = config.lock();
        let file = config[field].as_str().unwrap_or("");
        // file not found
        if fs::metadata(file).is_err() {
            // clear this field
            config[field] = Value::Null;
        }
    }
}
