use bp_cli::commands::generate;
use tide::http::mime;
use tokio::fs;

use crate::state::State;

pub struct SecurityController;

static CERT_FILE: &str = "cert.der";
static PRIV_KEY_FILE: &str = "key.der";

impl SecurityController {
    pub async fn query(_req: tide::Request<State>) -> tide::Result {
        let (cert, key) = tokio::join!(fs::metadata(CERT_FILE), fs::metadata(PRIV_KEY_FILE));

        let certificate = if cert.is_ok() { CERT_FILE } else { "" };
        let private_key = if key.is_ok() { PRIV_KEY_FILE } else { "" };

        let resp = serde_json::json!({
            "certificate": certificate,
            "private_key": private_key,
        });

        Ok(tide::Response::builder(200)
            .body(resp.to_string())
            .content_type(mime::JSON)
            .build())
    }

    pub async fn create(mut req: tide::Request<State>) -> tide::Result {
        #[derive(serde::Deserialize)]
        struct Params {
            hostname: String,
        }

        let Params { hostname } = req.body_json().await?;

        generate::generate_certificate(&hostname).await?;

        Self::query(req).await
    }
}
