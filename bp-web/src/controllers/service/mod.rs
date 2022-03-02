mod handle;

use bp_cli::commands::service;
use bp_core::{ClientOptions, Options, ServerOptions, Startup};
use handle::ServiceHandle;
use lazy_static::lazy_static;
use serde_json::json;
use tide::http::mime;
use tokio::sync::mpsc::channel;

use crate::{constants::DEFAULT_CONFIG_FILE, options::RunType, state::State};

lazy_static! {
    static ref SERVICE_HANDLE: ServiceHandle = Default::default();
}

pub struct ServiceController;

impl ServiceController {
    pub async fn query(_req: tide::Request<State>) -> tide::Result {
        let info = SERVICE_HANDLE.info();

        Ok(tide::Response::builder(200)
            .content_type(mime::JSON)
            .body(json!({ "service_info": info }))
            .build())
    }

    pub async fn start(req: tide::Request<State>) -> tide::Result {
        let state = req.state();

        let opts = match state.opts.run_type() {
            RunType::Client => Options::Client(ClientOptions {
                config: Some(DEFAULT_CONFIG_FILE.to_string()),
                ..Default::default()
            }),
            RunType::Server => Options::Server(ServerOptions {
                config: Some(DEFAULT_CONFIG_FILE.to_string()),
                ..Default::default()
            }),
        };

        let (startup_sender, mut startup_receiver) = channel::<Startup>(1);
        let (shutdown_sender, mut shutdown_receiver) = channel::<()>(1);

        tokio::spawn(async move {
            let _ = service::run(opts, startup_sender, shutdown_receiver.recv()).await;
            SERVICE_HANDLE.set(None);
        });

        let startup = startup_receiver.recv().await.unwrap();

        match startup {
            Startup::Fail(err) => Ok(tide::Response::builder(500).body(err.to_string()).build()),
            Startup::Success(info) => {
                SERVICE_HANDLE.set(Some((shutdown_sender, info)));
                Self::query(req).await
            }
        }
    }

    pub async fn stop(_req: tide::Request<State>) -> tide::Result {
        if !SERVICE_HANDLE.running() {
            return Ok(tide::Response::builder(500).body("service is not running").build());
        }

        SERVICE_HANDLE.abort().await;

        Ok(tide::Response::builder(200).build())
    }
}
