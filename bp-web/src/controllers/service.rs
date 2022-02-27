use bp_cli::commands::service;
use bp_core::{ClientOptions, Options, ServerOptions, ServiceInfo, Startup};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde_json::json;
use tide::http::mime;
use tokio::sync::mpsc::{channel, Sender};

use crate::{constants::DEFAULT_CONFIG_FILE, options::RunType, state::State};

type Service = (Sender<()>, ServiceInfo);

#[derive(Default)]
struct ServiceHandle {
    inner: Mutex<Option<Service>>,
}

impl ServiceHandle {
    fn info(&self) -> Option<ServiceInfo> {
        let inner = self.inner.lock();
        (*inner).as_ref().map(|(_, info)| info.clone())
    }

    fn sender(&self) -> Option<Sender<()>> {
        let inner = self.inner.lock();
        (*inner).as_ref().map(|(sender, _)| sender.clone())
    }

    fn set(&self, value: Service) {
        let mut inner = self.inner.lock();
        *inner = Some(value);
    }

    fn clear(&self) {
        let mut inner = self.inner.lock();
        *inner = None;
    }
}

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
            let shutdown_fut = async {
                shutdown_receiver.recv().await;
            };
            let _ = service::run(opts, startup_sender, shutdown_fut).await;
        });

        let startup = startup_receiver.recv().await.unwrap();

        match startup {
            Startup::Fail(err) => Ok(tide::Response::builder(500).body(err.to_string()).build()),
            Startup::Success(info) => {
                SERVICE_HANDLE.set((shutdown_sender, info));
                Self::query(req).await
            }
        }
    }

    pub async fn stop(_req: tide::Request<State>) -> tide::Result {
        let sender = SERVICE_HANDLE.sender();

        if sender.is_none() {
            return Ok(tide::Response::builder(500).body("service is not running").build());
        }

        sender.unwrap().send(()).await.unwrap();
        SERVICE_HANDLE.clear();

        Ok(tide::Response::builder(200).build())
    }
}
