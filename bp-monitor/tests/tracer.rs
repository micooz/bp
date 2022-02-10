use std::time::Duration;

use bp_monitor::{
    events,
    sender::{Sender, UdpSender},
    tracer::Tracer,
};
use tokio::time::sleep;

#[tokio::test(flavor = "multi_thread")]
async fn test_tracer() {
    let mut sender = UdpSender::default();
    sender.subscribe("127.0.0.1:1234".parse().unwrap());
    sender.subscribe("127.0.0.1:4567".parse().unwrap());

    let mut tracer = Tracer::default();
    tracer.add_sender(Box::new(sender)).await;
    tracer.log(events::NewIncomingConnectionEvent { inner: 100 });

    sleep(Duration::from_secs(1)).await;
}
