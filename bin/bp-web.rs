use bp_core::logging;
use bp_web::{run, Options};
use clap::StructOpt;

#[tokio::main]
async fn main() {
    logging::init().unwrap();

    let opts = Options::parse();
    run(opts).await;
}
