use bp_core::*;

#[test]
fn test_from_file() {
    let opts = Options::from_file("tests/fixtures/config.yml").unwrap();
    assert_eq!(opts.client, true);
    assert_eq!(opts.daemonize, true);
    assert_eq!(opts.bind, "127.0.0.1:1080".parse().unwrap());
    assert_eq!(opts.protocol, ApplicationProtocol::EncryptRandomPadding);

    let opts = Options::from_file("tests/fixtures/config.yaml").unwrap();
    assert_eq!(opts.client, true);

    let opts = Options::from_file("tests/fixtures/config.json").unwrap();
    assert_eq!(opts.server, true);
    assert_eq!(opts.bind, "127.0.0.1:1080".parse().unwrap());
    assert_eq!(opts.key, Some("key".to_string()));

    assert!(Options::from_file("tests/fixtures/config.invalid").is_err());
}

#[test]
fn test_service_type() {
    let opts = Options {
        client: true,
        ..Options::default()
    };

    assert!(matches!(opts.service_type(), ServiceType::Client));

    let opts = Options {
        server: true,
        ..Options::default()
    };

    assert!(matches!(opts.service_type(), ServiceType::Server));
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
fn test_set_client_and_server() {
    let opts = Options {
        client: true,
        server: true,
        ..Options::default()
    };

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
        key: Some("key".to_string()),
        proxy_white_list: Some("/tmp/proxy_white_list.txt".to_string()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_proxy_white_list_is_set_empty() {
    let opts = Options {
        client: true,
        proxy_white_list: Some("".to_string()),
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
        key: Some("key".to_string()),
        udp_over_tcp: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_force_dest_addr() {
    let opts = Options {
        server: true,
        key: Some("key".to_string()),
        force_dest_addr: Some("example.com:443".parse().unwrap()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_client_and_quic() {
    let opts = Options {
        client: true,
        quic: true,
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_quic() {
    let opts = Options {
        server: true,
        key: Some("key".to_string()),
        quic: true,
        tls_cert: Some("cert.pem".to_string()),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_quic_max_concurrency() {
    let opts = Options {
        server: true,
        key: Some("key".to_string()),
        quic_max_concurrency: Some(0),
        ..Options::default()
    };

    assert!(check_options(&opts).is_err());
}
