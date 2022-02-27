use bp_cli::utils::exit::{exit, ExitError};

use crate::{constants::DEFAULT_BIND_ADDRESS, routes, state::State, Options};

pub async fn run(opts: Options) {
    if let Err(err) = opts.check() {
        log::error!("{}", err);
        exit(ExitError::ArgumentsError);
    }
    if let Err(err) = bootstrap(opts).await {
        log::error!("bootstrap failed due to: {}", err);
    }
}

async fn bootstrap(opts: Options) -> tide::Result<()> {
    let bind_addr = opts.bind.clone().unwrap_or_else(|| DEFAULT_BIND_ADDRESS.to_string());

    // init shared state
    let state = State { opts };

    // create web server and register routes
    let mut app = tide::with_state(state);
    routes::register(&mut app);

    // start listening
    app.listen(bind_addr).await?;

    Ok(())
}
