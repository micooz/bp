use std::time::Duration;

use bp_monitor::{events, tracer::Tracer};
use tokio::time::sleep;

#[tokio::test(flavor = "multi_thread")]
async fn test_tracer() {
    let mut tracer = Tracer::new();
    tracer.init().await;

    tracer.add_subscriber("127.0.0.1:1234".parse().unwrap()).await;
    tracer.add_subscriber("127.0.0.1:4567".parse().unwrap()).await;

    tracer.log(events::NewConnectionEvent { inner: 100 });

    sleep(Duration::from_secs(1)).await;
}
