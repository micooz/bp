use regex::Regex;
use std::{
    collections::HashSet,
    fs,
    io::Read,
    net::{IpAddr, SocketAddr, TcpStream},
    result::Result,
    sync::{Arc, Mutex},
    time::Duration,
};
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    error::ResolveError,
    TokioAsyncResolver,
};

lazy_static::lazy_static! {
    static ref GOOGLE_RESOLVER: TokioAsyncResolver = {
        let config = ResolverConfig::default(); // google

        let mut opts = ResolverOpts::default();
        // opts.timeout = Duration::from_secs(2);
        opts.attempts = 1;

        TokioAsyncResolver::tokio(config, opts).unwrap()
    };
    static ref ANOTHER_RESOLVER: TokioAsyncResolver = {
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: "202.38.93.153:5353".parse().unwrap(),
            protocol: Protocol::Tcp,
            tls_dns_name: None,
            trust_nx_responses: false,
        });

        let mut opts = ResolverOpts::default();
        // opts.timeout = Duration::from_secs(2);
        opts.attempts = 1;

        TokioAsyncResolver::tokio(config, opts).unwrap()
    };
}

const BASE64_ENCODED_GFW_LIST_FILE: &str = "assets/gfwlist.txt";
const DOMAIN_LIST_FILE: &str = "assets/domains.txt";
const HOST_REGULAR_EXPRESSION: &str = r"[^\-!|/\.]([\w\-]+\.[\.\-\w]+)";
const CONNECT_TIMEOUT_SECONDS: u64 = 10;
const THREAD_COUNT: usize = 8;

#[derive(Debug)]
enum ResolverType {
    Google,
    Another,
}

#[derive(Default, Debug)]
struct Summary {
    pub total_tests: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub retry_success_count: usize,
    pub retry_fail_count: usize,
}

#[tokio::main]
async fn main() {
    let mut domains = Vec::with_capacity(7000);

    match fs::OpenOptions::new().read(true).open(DOMAIN_LIST_FILE) {
        Ok(mut f) => {
            let mut buffer = String::new();
            f.read_to_string(&mut buffer).unwrap();

            let mut lines = buffer.lines().map(|s| s.to_string()).collect::<Vec<String>>();
            domains.append(&mut lines);
        }
        Err(_) => {
            let mut lines = get_domains();
            domains.append(&mut lines);
            fs::write(DOMAIN_LIST_FILE, domains.join("\n")).unwrap();
        }
    };

    // take the first N elements
    let _ = domains.split_off(10);

    let summary = Arc::new(Mutex::new(Summary::default()));
    let chunks = get_chunks(&domains, domains.len() / THREAD_COUNT);

    let mut handlers = vec![];

    for chunk in chunks {
        let summary = summary.clone();
        handlers.push(tokio::spawn(async move {
            thread_worker(summary, chunk.to_vec()).await;
        }));
    }

    for handler in handlers {
        handler.await.unwrap();
    }

    let summary = summary.lock().unwrap();

    println!("\n");
    println!("{:?}", summary);
}

async fn thread_worker(summary: Arc<Mutex<Summary>>, domains: Vec<String>) {
    for domain in domains.iter() {
        let mut last_ip: Option<IpAddr> = None;
        let mut is_retry = false;
        let mut resolver_type = ResolverType::Google;

        let mut logs = vec![];

        loop {
            logs.push(format!("> [{:?}] resolving {}", resolver_type, domain));

            match dns_resolve(&resolver_type, domain).await {
                Ok(ip) => {
                    logs.push(format!("  | {} -> {}", domain, ip));

                    if last_ip == Some(ip) {
                        logs.push(format!("  | got the same ip, skipped"));
                        break;
                    } else {
                        last_ip = Some(ip);
                    }

                    logs.push(format!("  | connecting {}({})", domain, ip));

                    if detect_tcp_connectivity(ip) {
                        logs.push(format!("  | successfully"));
                        let mut summary = summary.lock().unwrap();
                        if is_retry {
                            summary.retry_success_count += 1;
                        } else {
                            summary.success_count += 1;
                        }
                        break;
                    } else {
                        logs.push(format!("  | fail"));
                        let mut summary = summary.lock().unwrap();
                        if is_retry {
                            summary.retry_fail_count += 1;
                            break;
                        } else {
                            // retry
                            logs.push(format!("  | retry another dns server..."));
                            summary.fail_count += 1;
                            is_retry = true;
                            resolver_type = ResolverType::Another;
                        }
                    }
                }
                Err(err) => {
                    logs.push(format!("< fail to resolve {} due to {}", domain, err));
                    let mut summary = summary.lock().unwrap();
                    if is_retry {
                        summary.retry_fail_count += 1;
                    } else {
                        summary.fail_count += 1;
                    }
                    break;
                }
            }
        }

        summary.lock().unwrap().total_tests += 1;

        println!("{}", logs.join("\n"));
    }
}

fn get_domains() -> Vec<String> {
    let mut domains = HashSet::with_capacity(7000);

    // decode base64 string
    let base64_buf = fs::read(BASE64_ENCODED_GFW_LIST_FILE).unwrap();
    // drop all '\n'
    let base64_buf = base64_buf.into_iter().filter(|c| *c != b'\n').collect::<Vec<u8>>();

    let base64_decoded_buf = base64::decode(base64_buf).unwrap();

    // parsed with utf8 encoding
    let utf8_text = String::from_utf8(base64_decoded_buf).unwrap();

    // regex match
    let re = Regex::new(HOST_REGULAR_EXPRESSION).unwrap();
    let matches = re.captures_iter(&utf8_text);

    for cap in matches.into_iter() {
        let item = cap.get(0).unwrap();
        // skip ip address
        if item.as_str().parse::<IpAddr>().is_ok() {
            continue;
        }
        domains.insert(item.as_str().trim().to_string());
    }

    domains.into_iter().collect()
}

fn get_chunks<T: Clone>(arr: &Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
    let mut chunks = vec![];
    let mut ptr = 0;

    if chunk_size == 0 {
        panic!("chunk_size should be greater than 0");
    }

    loop {
        let end = std::cmp::min(arr.len(), ptr + chunk_size);
        let chunk = arr[ptr..end].to_vec();
        let chunk_len = chunk.len();

        chunks.push(chunk);

        if chunk_len < chunk_size {
            break;
        }

        ptr += chunk_size;
    }

    chunks
}

async fn dns_resolve(resolver_type: &ResolverType, domain: &str) -> Result<IpAddr, ResolveError> {
    let ip = match resolver_type {
        ResolverType::Google => GOOGLE_RESOLVER.lookup_ip(domain).await?,
        ResolverType::Another => ANOTHER_RESOLVER.lookup_ip(domain).await?,
    };
    Ok(ip.into_iter().next().unwrap())
}

fn detect_tcp_connectivity(ip: IpAddr) -> bool {
    let addr = SocketAddr::from((ip, 80));
    TcpStream::connect_timeout(&addr, Duration::from_secs(CONNECT_TIMEOUT_SECONDS)).is_ok()
}

#[test]
fn test_dedup_string() {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    let mut strings = ["github.com", "lgpl-2.1.txt", "github.com"].to_vec();
    strings.dedup();
    assert_eq!(strings.len(), 3);

    let mut strings = ["github.com", "github.com", "lgpl-2.1.txt"].to_vec();
    strings.dedup();
    assert_eq!(strings.len(), 2);

    let strings: HashSet<&str> = HashSet::from_iter(["github.com", "github.com", "lgpl-2.1.txt"]);
    assert_eq!(strings.len(), 2);
}
