use std::{fs, sync::Arc};

use anyhow::Result;
use bp_cli::{commands::generate, options::generate::ConfigType};
use bp_core::{ClientOptions, Options, ServerOptions};
use parking_lot::Mutex;
use serde::Deserialize;
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

        let resp_empty = json!({
            "file_path": null,
            "config": null,
            "metadata": null,
        });

        let file = tokio::fs::read_to_string(DEFAULT_CONFIG_FILE).await;
        if file.is_err() {
            return Ok(tide::Response::builder(200)
                .body(resp_empty)
                .content_type(mime::JSON)
                .build());
        }

        // parse
        let config: Result<Value, Error> = from_str(&file.unwrap());
        if config.is_err() {
            return Ok(tide::Response::builder(200)
                .body(resp_empty)
                .content_type(mime::JSON)
                .build());
        }

        let config = Arc::new(Mutex::new(config.unwrap()));

        // check existence of tls_cert, tls_key and acl
        Self::check_config_field(config.clone(), "tls_cert");
        Self::check_config_field(config.clone(), "acl");

        if state.opts.server {
            Self::check_config_field(config.clone(), "tls_key");
        }

        let resp = json!({
            "file_path": DEFAULT_CONFIG_FILE,
            "config": *config.lock(),
            "metadata": null, // TODO: "metadata": Cli::metadata(),
        });

        Ok(tide::Response::builder(200).body(resp).content_type(mime::JSON).build())
    }

    pub async fn query_acl(req: tide::Request<State>) -> tide::Result {
        let mut content = "".to_string();
        let mut file_path = "".to_string();

        if let Ok(config) = Self::get_config(req.state()) {
            if let Some(acl) = config.acl() {
                let file = tokio::fs::read_to_string(&acl).await;
                content = file.unwrap_or_else(|_| "".to_string());
                file_path = acl;
            }
        }

        Ok(tide::Response::builder(200)
            .body(json!({ "file_path": file_path, "content": content }))
            .content_type(mime::JSON)
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
        #[derive(Deserialize)]
        struct Params {
            modify_type: String,
            content: String,
        }

        let Params { modify_type, content } = req.body_json::<Params>().await?;

        if content.is_empty() {
            return Ok(tide::Response::builder(403)
                .body("content cannot be empty".to_string())
                .build());
        }

        let config = Self::get_config(req.state());

        if config.is_err() {
            return Ok(tide::Response::builder(403).build());
        }

        let config = config.unwrap();

        let acl_file = config.acl().unwrap_or_else(|| DEFAULT_ACL_FILE.to_string());

        let file = match modify_type.as_str() {
            "config" => DEFAULT_CONFIG_FILE,
            "acl" => acl_file.as_str(),
            _ => return Ok(tide::Response::builder(403).build()),
        };

        tokio::fs::write(file, content).await?;

        Ok(tide::Response::builder(200).build())
    }

    fn get_config(state: &State) -> Result<Options> {
        if state.opts.client {
            let mut opts = Options::Client(ClientOptions::default());
            opts.try_load_from_file(DEFAULT_CONFIG_FILE)?;
            return Ok(opts);
        }
        if state.opts.server {
            let mut opts = Options::Server(ServerOptions::default());
            opts.try_load_from_file(DEFAULT_CONFIG_FILE)?;
            return Ok(opts);
        }
        unreachable!()
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
