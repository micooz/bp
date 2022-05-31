mod handle;

use bp_core::{ClientOptions, Options, ServerOptions, Startup};
use handle::ServiceHandle;
use lazy_static::lazy_static;
use serde_json::json;
use tokio::sync::mpsc::channel;

use crate::{
    commands::service,
    options::web::RunType,
    web::{
        common::{response::Response, state::State},
        utils::finder::find_config_path,
    },
};

lazy_static! {
    static ref SERVICE_HANDLE: ServiceHandle = Default::default();
}

pub struct ServiceController;

impl ServiceController {
    pub async fn query(_req: tide::Request<State>) -> tide::Result {
        let info = SERVICE_HANDLE.info();

        Response::success(json!(info))
    }

    pub async fn start(req: tide::Request<State>) -> tide::Result {
        let state = req.state();
        let config_path = find_config_path();

        let opts = match state.opts.run_type() {
            RunType::Client => Options::Client(ClientOptions {
                config: Some(config_path),
                ..Default::default()
            }),
            RunType::Server => Options::Server(ServerOptions {
                config: Some(config_path),
                ..Default::default()
            }),
        };

        let (startup_sender, mut startup_receiver) = channel::<Startup>(1);
        let (shutdown_sender, mut shutdown_receiver) = channel::<()>(1);

        tokio::spawn(async move {
            service::run(opts, startup_sender, shutdown_receiver.recv()).await;
            SERVICE_HANDLE.set(None);
        });

        let startup = startup_receiver.recv().await.unwrap();

        match startup {
            Startup::Fail(err) => Response::error(500, err.to_string().as_str()),
            Startup::Success(info) => {
                SERVICE_HANDLE.set(Some((shutdown_sender, info)));
                Self::query(req).await
            }
        }
    }

    pub async fn stop(_req: tide::Request<State>) -> tide::Result {
        if !SERVICE_HANDLE.running() {
            return Response::error(500, "service is not running");
        }

        SERVICE_HANDLE.abort().await;

        Response::success(serde_json::Value::Null)
    }
}
