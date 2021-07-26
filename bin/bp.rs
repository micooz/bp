use bp::net::AcceptResult;
use bp::net::{connection::Connection, service};
use bp::options::Options;
use clap::Clap;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts: Options = Clap::parse();

    // check -c or -s
    if opts.client == false && opts.server == false {
        log::error!("-c or -s must be set");
        return;
    }
    if opts.client == true && opts.server == true {
        log::error!("-c or -s can only be set one");
        return;
    }

    // check --remote-host and --remote-port
    if opts.client == true && (opts.remote_host == None || opts.remote_port == None) {
        if opts.remote_host == None {
            log::error!("--remote-host must be set when specify -c");
        }
        if opts.remote_port == None {
            log::error!("--remote-port must be set when specify -c");
        }
        return;
    }

    let (tx, mut rx) = mpsc::channel::<AcceptResult>(32);

    let local_addr = opts.get_local_addr();

    // start local service
    tokio::spawn(async move {
        service::bootstrap(local_addr, tx).await;
    });

    // handle connections
    while let Some(accept) = rx.recv().await {
        let addr = accept.socket.peer_addr().unwrap();
        let mut conn = Connection::new(accept.socket, opts.clone());
        let service_type = opts.get_service_type().unwrap();

        tokio::spawn(async move {
            log::info!("[{}] connected", addr);

            match conn.handle(service_type).await {
                Ok(_) => {}
                Err(err) => {
                    log::error!("{:?}", err);
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                }
            }

            log::info!("[{}] disconnected, in/out = {}/{}", addr, 0, 0);
        });
    }
}
