use std::sync::Arc;

use anyhow::Result;
use bp_core::{ClientOptions, Options, ServerOptions};
use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::{from_str, json, Value};
use tokio::fs;

use crate::{
    commands::generate,
    options::{generate::ConfigType, web::RunType},
    web::{
        common::{response::Response, state::State},
        constants::{DEFAULT_ACL_FILE, DEFAULT_CERT_FILE, DEFAULT_CONFIG_FILE, DEFAULT_PRIV_KEY_FILE},
        utils::finder::find_config_path,
    },
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

        let file_path = find_config_path();
        let file = fs::read_to_string(&file_path).await;

        if file.is_err() {
            return Response::success(resp_empty);
        }

        // parse
        let config: Result<Value, serde_json::Error> = from_str(&file.unwrap());

        if config.is_err() {
            return Response::success(resp_empty);
        }

        let config = Arc::new(Mutex::new(config.unwrap()));

        // check existence of tls_cert, tls_key and acl
        Self::check_config_field(config.clone(), "tls_cert");
        Self::check_config_field(config.clone(), "acl");

        if state.opts.server {
            Self::check_config_field(config.clone(), "tls_key");
        }

        let resp = json!({
            "file_path": file_path,
            "config": *config.lock(),
            "metadata": null, // TODO: "metadata": Cli::metadata(),
        });

        Response::success(resp)
    }

    pub async fn query_acl(req: tide::Request<State>) -> tide::Result {
        let mut content = "".to_string();
        let mut file_path = "".to_string();

        if let Ok(config) = Self::get_config(req.state()) {
            if let Some(acl) = config.acl() {
                let file = fs::read_to_string(&acl).await;
                content = file.unwrap_or_else(|_| "".to_string());
                file_path = acl;
            }
        }

        Response::success(json!({ "file_path": file_path, "content": content }))
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

        Response::success(Value::Null)
    }

    pub async fn modify(mut req: tide::Request<State>) -> tide::Result {
        #[derive(Deserialize)]
        struct Params {
            modify_type: String,
            content: String,
        }

        let Params { modify_type, content } = req.body_json::<Params>().await?;

        if content.is_empty() {
            return Response::error(403, "content cannot be empty");
        }

        let config = Self::get_config(req.state());

        if config.is_err() {
            return Response::error(403, "");
        }

        let config = config.unwrap();

        let acl_file = config.acl().unwrap_or_else(|| DEFAULT_ACL_FILE.to_string());
        let config_file = find_config_path();

        let file = match modify_type.as_str() {
            "config" => config_file.as_str(),
            "acl" => acl_file.as_str(),
            _ => return Response::error(403, ""),
        };

        fs::write(file, content).await?;

        Response::success(Value::Null)
    }

    fn get_config(state: &State) -> Result<Options> {
        let path = find_config_path();

        if state.opts.client {
            let mut opts = Options::Client(ClientOptions::default());
            opts.try_load_from_file(path.as_str())?;
            return Ok(opts);
        }
        if state.opts.server {
            let mut opts = Options::Server(ServerOptions::default());
            opts.try_load_from_file(path.as_str())?;
            return Ok(opts);
        }

        unreachable!()
    }

    fn check_config_field(config: Arc<Mutex<Value>>, field: &'static str) {
        let mut config = config.lock();
        let file = config[field].as_str().unwrap_or("");
        // file not found
        if std::fs::metadata(file).is_err() {
            // clear this field
            config[field] = Value::Null;
        }
    }
}
