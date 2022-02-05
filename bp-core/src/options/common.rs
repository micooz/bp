use anyhow::Result;

use crate::{constants, options_from_file, Address, ClientOptions, EncryptionMethod, ServerOptions};

#[derive(Clone, Copy)]
pub enum ServiceType {
    Client,
    Server,
}

#[derive(Clone)]
pub enum Options {
    Client(ClientOptions),
    Server(ServerOptions),
}

impl Options {
    pub fn try_load_from_file(&mut self, config: &str) -> Result<()> {
        match self.service_type() {
            ServiceType::Client => {
                let file_opts = options_from_file::<ClientOptions>(config)?;
                *self = Self::Client(file_opts);
            }
            ServiceType::Server => {
                let file_opts = options_from_file::<ServerOptions>(config)?;
                *self = Self::Server(file_opts);
            }
        }
        Ok(())
    }

    pub fn check(&self) -> Result<()> {
        match self {
            Self::Client(opts) => opts.check(),
            Self::Server(opts) => opts.check(),
        }
    }

    pub fn set_bind(&mut self, bind: Address) {
        match self {
            Self::Client(opts) => opts.bind = bind,
            Self::Server(opts) => opts.bind = bind,
        }
    }
}

impl Options {
    pub fn is_client(&self) -> bool {
        matches!(self, Self::Client { .. })
    }

    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server { .. })
    }

    pub fn config(&self) -> Option<String> {
        match self {
            Self::Client(opts) => opts.config.clone(),
            Self::Server(opts) => opts.config.clone(),
        }
    }

    pub fn service_type(&self) -> ServiceType {
        match self {
            Self::Client(_) => ServiceType::Client,
            Self::Server(_) => ServiceType::Server,
        }
    }

    pub fn key(&self) -> String {
        match self {
            Self::Client(opts) => opts.key.clone().unwrap(),
            Self::Server(opts) => opts.key.clone(),
        }
    }

    pub fn dns_server(&self) -> Address {
        let server = match self {
            Self::Client(opts) => opts.dns_server.clone(),
            Self::Server(opts) => opts.dns_server.clone(),
        };
        server.unwrap_or_else(|| constants::DEFAULT_DNS_SERVER_ADDRESS.parse().unwrap())
    }

    pub fn udp_over_tcp(&self) -> bool {
        if let Self::Client(opts) = self {
            return opts.udp_over_tcp;
        }
        unreachable!()
    }

    pub fn proxy_white_list(&self) -> Option<String> {
        if let Self::Client(opts) = self {
            return opts.proxy_white_list.clone();
        }
        unreachable!()
    }

    pub fn force_dest_addr(&self) -> Option<Address> {
        if let Self::Client(opts) = self {
            return opts.force_dest_addr.clone();
        }
        unreachable!()
    }

    pub fn quic(&self) -> bool {
        match self {
            Self::Client(opts) => opts.quic,
            Self::Server(opts) => opts.quic,
        }
    }

    pub fn tls(&self) -> bool {
        match self {
            Self::Client(opts) => opts.tls,
            Self::Server(opts) => opts.tls,
        }
    }

    pub fn quic_max_concurrency(&self) -> Option<u16> {
        if let Self::Client(opts) = self {
            return opts.quic_max_concurrency;
        }
        unreachable!()
    }

    pub fn bind(&self) -> Address {
        match self {
            Self::Client(opts) => opts.bind.clone(),
            Self::Server(opts) => opts.bind.clone(),
        }
    }

    pub fn pac_bind(&self) -> Option<Address> {
        if let Self::Client(opts) = self {
            return opts.pac_bind.clone();
        }
        unreachable!()
    }

    pub fn server_bind(&self) -> Option<Address> {
        if let Self::Client(opts) = self {
            return opts.server_bind.clone();
        }
        unreachable!()
    }

    pub fn encryption(&self) -> EncryptionMethod {
        match self {
            Self::Client(opts) => opts.encryption,
            Self::Server(opts) => opts.encryption,
        }
    }

    pub fn tls_cert(&self) -> Option<String> {
        match self {
            Self::Client(opts) => opts.tls_cert.clone(),
            Self::Server(opts) => opts.tls_cert.clone(),
        }
    }

    pub fn tls_key(&self) -> Option<String> {
        if let Self::Server(opts) = self {
            return opts.tls_key.clone();
        }
        unreachable!()
    }
}
