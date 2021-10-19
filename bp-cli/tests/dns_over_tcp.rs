use bp_cli::test_utils::run_bp;
use bp_core::{Options, StartupInfo};
use bp_test::send_recv::udp_oneshot;

#[tokio::test(flavor = "multi_thread")]
async fn test_dns_over_tcp() {
    let opts = Options {
        client: true,
        enable_udp: true,
        dns_over_tcp: true,
        ..Default::default()
    };

    let StartupInfo { bind_addr, .. } = run_bp(opts).await;

    let buf = udp_oneshot(&bind_addr, include_bytes!("fixtures/normal_dns_query.bin")).await;

    assert!(!buf.is_empty());
}
