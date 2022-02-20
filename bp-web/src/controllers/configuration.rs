use bp_cli::commands::generate;
use tide::http::mime;
use tokio::fs;

pub struct ConfigurationController;

const CONFIG_FILE: &str = "config.json";

impl ConfigurationController {
    pub async fn query(_req: tide::Request<()>) -> tide::Result {
        let file = fs::read_to_string(CONFIG_FILE).await;

        if file.is_err() {
            return Ok(tide::Response::builder(404).build());
        }

        Ok(tide::Response::builder(200)
            .body(file.unwrap())
            .content_type(mime::JSON)
            .build())
    }

    pub async fn create(req: tide::Request<()>) -> tide::Result {
        let res = generate::generate_config(CONFIG_FILE, bp_cli::options::generate::ConfigType::Client).await;

        if res.is_err() {
            return Ok(tide::Response::builder(500).build());
        }

        Self::query(req).await
    }

    pub async fn modify(mut req: tide::Request<()>) -> tide::Result {
        let new_content = req.body_string().await?;

        let res = fs::write(CONFIG_FILE, new_content).await;

        if res.is_err() {
            return Ok(tide::Response::builder(500).build());
        }

        Self::query(req).await
    }
}
