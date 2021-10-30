use bp_core::*;

#[test]
fn test_from_file() {
    let opts = Options::from_file("tests/fixtures/config.yml").unwrap();
    assert_eq!(opts.client, true);
    assert_eq!(opts.daemonize, true);
    assert_eq!(opts.bind, "127.0.0.1:1080".parse().unwrap());
    assert_eq!(opts.protocol, TransportProtocol::EncryptRandomPadding);

    let opts = Options::from_file("tests/fixtures/config.json").unwrap();
    assert_eq!(opts.server, true);
    assert_eq!(opts.bind, "127.0.0.1:1080".parse().unwrap());
    assert_eq!(opts.key, Some("key".to_string()));
}

#[test]
fn test_service_type() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    assert!(matches!(opts.service_type(), ServiceType::Client));
}

#[test]
fn test_get_dns_server() {
    let opts = Options::default();

    assert_eq!(opts.get_dns_server(), "8.8.8.8:53".parse().unwrap());
}

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
fn test_set_client_and_udp_over_tcp() {
    let opts = Options {
        client: true,
        udp_over_tcp: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_udp_over_tcp() {
    let opts = Options {
        server: true,
        udp_over_tcp: true,
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
