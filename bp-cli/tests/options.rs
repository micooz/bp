use bp_cli::{check_options, Options};

#[test]
fn test_empty() {
    let opts = Options::default();

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_client() {
    let opts = Options {
        client: true,
        ..Default::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_client_and_server_host_port() {
    let opts = Options {
        client: true,
        server_bind: Some("localhost:8888".to_string()),
        ..Default::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_client_and_server_host_port_and_key() {
    let opts = Options {
        client: true,
        server_bind: Some("localhost:8888".to_string()),
        key: Some("key".to_string()),
        ..Default::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_server() {
    let opts = Options {
        server: true,
        ..Default::default()
    };

    assert!(check_options(&opts).is_err());
}

#[test]
fn test_set_server_and_key() {
    let opts = Options {
        server: true,
        key: Some("key".to_string()),
        ..Default::default()
    };

    assert!(check_options(&opts).is_ok());
}

#[test]
fn test_set_server_and_proxy_list_path() {
    let opts = Options {
        server: true,
        proxy_list_path: Some("/tmp/proxy_list.txt".to_string()),
        ..Default::default()
    };

    assert!(check_options(&opts).is_err());
}