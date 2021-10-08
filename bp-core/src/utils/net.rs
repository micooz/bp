use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use std::result::Result;
use tokio::net::UdpSocket;

pub async fn create_udp_client_with_random_port() -> Result<UdpSocket, &'static str> {
    let mut max_retry_times = 10u8;
    let mut rng = StdRng::from_rng(thread_rng()).unwrap();

    loop {
        // TODO: allow custom port range
        let port: u32 = rng.gen_range(10000..65535);
        let bind_addr = format!("0.0.0.0:{}", port);

        match UdpSocket::bind(bind_addr).await {
            Ok(socket) => {
                return Ok(socket);
            }
            Err(_) => {
                max_retry_times -= 1;

                if max_retry_times == 0 {
                    return Err("udp socket random bind error, max retry times exceed");
                }
            }
        }
    }
}
