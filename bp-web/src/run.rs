// use serde::Deserialize;

// #[derive(Debug, Deserialize)]
// struct Animal {
//     name: String,
//     legs: u8,
// }
// async fn order_shoes(req: Request<()>) -> tide::Result {
//     let Animal { name, legs } = req.query()?;
//     Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
// }

use anyhow::Result;
use tokio::fs;

use crate::Options;

static INDEX_HTML_CONTENT: &[u8] = include_bytes!("../web/dist/index.html");
static INDEX_JS_CONTENT: &[u8] = include_bytes!("../web/dist/index.js");

pub async fn run(opts: Options) {
    if let Err(err) = bootstrap(opts).await {
        log::error!("bootstrap failed due to: {}", err);
    }
}

async fn bootstrap(opts: Options) -> tide::Result<()> {
    expand_files().await?;

    let bind_addr = opts.bind.unwrap_or("127.0.0.1:8080".to_string());

    let mut app = tide::new();

    // register routers
    app.at("/").serve_file("index.html")?;

    // start listening
    app.listen(bind_addr).await?;

    Ok(())
}

async fn expand_files() -> Result<()> {
    fs::write("index.html", INDEX_HTML_CONTENT).await?;
    fs::write("index.js", INDEX_JS_CONTENT).await?;
    Ok(())
}
