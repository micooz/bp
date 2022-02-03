#[cfg(test)]
mod test_utils {
    use bp_core::{options_from_file, ClientOptions, ServerOptions};

    #[test]
    fn test_from_file() {
        assert!(options_from_file::<ClientOptions>("tests/fixtures/config.yml").is_ok());
        assert!(options_from_file::<ClientOptions>("tests/fixtures/config.yaml").is_ok());
        assert!(options_from_file::<ServerOptions>("tests/fixtures/config.json").is_ok());
        assert!(options_from_file::<ServerOptions>("tests/fixtures/config.invalid").is_err());
    }
}

#[cfg(test)]
mod test_cli {}

#[cfg(test)]
mod test_common {
    use bp_core::{ClientOptions, Options, ServerOptions, ServiceType};

    #[test]
    fn test_service_options() {
        let opts = Options::Client(ClientOptions::default());
        assert!(opts.is_client());
        assert!(matches!(opts.service_type(), ServiceType::Client));

        let opts = Options::Server(ServerOptions::default());
        assert!(opts.is_server());
        assert!(matches!(opts.service_type(), ServiceType::Server));
    }
}

#[cfg(test)]
mod test_client {
    use bp_core::{ClientOptions, OptionsChecker};

    #[test]
    fn test_checker() {
        let opts = ClientOptions::default();
        assert!(opts.check().is_ok());

        let opts = ClientOptions {
            server_bind: Some("127.0.0.1:1081".parse().unwrap()),
            ..Default::default()
        };
        assert!(opts.check().is_err());

        let opts = ClientOptions {
            udp_over_tcp: true,
            ..Default::default()
        };
        assert!(opts.check().is_err());

        let opts = ClientOptions {
            tls: true,
            quic: true,
            ..Default::default()
        };
        assert!(opts.check().is_err());

        let opts = ClientOptions {
            quic: true,
            ..Default::default()
        };
        assert!(opts.check().is_err());

        let opts = ClientOptions {
            quic_max_concurrency: Some(0),
            ..Default::default()
        };
        assert!(opts.check().is_err());
    }
}

#[cfg(test)]
mod test_server {
    use bp_core::{OptionsChecker, ServerOptions};

    #[test]
    fn test_checker() {
        let opts = ServerOptions::default();
        assert!(opts.check().is_ok());

        let opts = ServerOptions {
            tls: true,
            quic: true,
            ..Default::default()
        };
        assert!(opts.check().is_err());

        let mut opts = ServerOptions {
            quic: true,
            ..Default::default()
        };
        assert!(opts.check().is_err());

        opts.tls_cert = Some("cert.der".to_string());
        assert!(opts.check().is_err());

        opts.tls_key = Some("key.der".to_string());
        assert!(opts.check().is_ok());
    }
}
