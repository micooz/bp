use crate::{routes, Options};

pub async fn run(opts: Options) {
    if let Err(err) = bootstrap(opts).await {
        log::error!("bootstrap failed due to: {}", err);
    }
}

async fn bootstrap(opts: Options) -> tide::Result<()> {
    let bind_addr = opts.bind.unwrap_or("127.0.0.1:8080".to_string());

    let mut app = tide::new();

    // register routers
    app.at("/").get(routes::assets);
    app.at("/index.html").get(routes::assets);
    app.at("/static").get(routes::assets);

    app.at("/api/*").get(|_| async { Ok("Hello, world!") });

    // start listening
    app.listen(bind_addr).await?;

    Ok(())
}
