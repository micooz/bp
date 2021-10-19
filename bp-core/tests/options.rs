use bp_core::*;

#[test]
fn test_empty() {
    let opts = Options::default();

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_client() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_client_and_server_host_port() {
    let opts = Options {
        client: true,
        server_bind: Some("localhost:8888".parse().unwrap()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_client_and_server_host_port_and_key() {
    let opts = Options {
        client: true,
        server_bind: Some("localhost:8888".parse().unwrap()),
        key: Some("key".to_string()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_server() {
    let opts = Options {
        server: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_key() {
    let opts = Options {
        server: true,
        key: Some("key".to_string()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_server_and_proxy_white_list() {
    let opts = Options {
        server: true,
        proxy_white_list: Some("/tmp/proxy_white_list.txt".to_string()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_dns_over_tcp() {
    let opts = Options {
        server: true,
        dns_over_tcp: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_force_dest_addr() {
    let opts = Options {
        server: true,
        force_dest_addr: Some("example.com:443".parse().unwrap()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}
