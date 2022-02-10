use anyhow::Result;

use crate::{options_from_file, Address, ClientOptions, EncryptionMethod, ServerOptions};

#[derive(Clone, Copy)]
pub enum ServiceType {
    Client,
    Server,
}

#[derive(Debug, Clone)]
pub enum Options {
    Client(ClientOptions),
    Server(ServerOptions),
}

impl Options {
    pub fn try_load_from_file(&mut self, config: &str) -> Result<()> {
        match self {
            Self::Client(_) => {
                let file_opts = options_from_file::<ClientOptions>(config)?;
                *self = Self::Client(file_opts);
            }
            Self::Server(_) => {
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

    pub fn is_client(&self) -> bool {
        matches!(self, Self::Client { .. })
    }

    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server { .. })
    }

    pub fn service_type(&self) -> ServiceType {
        match self {
            Self::Client(_) => ServiceType::Client,
            Self::Server(_) => ServiceType::Server,
        }
    }

    pub fn client_opts(&self) -> ClientOptions {
        if let Self::Client(opts) = self {
            return opts.clone();
        }
        unreachable!()
    }

    pub fn server_opts(&self) -> ServerOptions {
        if let Self::Server(opts) = self {
            return opts.clone();
        }
        unreachable!()
    }
}

// aggregate common options
impl Options {
    pub fn config(&self) -> Option<String> {
        match self {
            Self::Client(opts) => opts.config.clone(),
            Self::Server(opts) => opts.config.clone(),
        }
    }

    pub fn key(&self) -> String {
        match self {
            Self::Client(opts) => opts.key.clone().unwrap(),
            Self::Server(opts) => opts.key.clone().unwrap(),
        }
    }

    pub fn acl(&self) -> Option<String> {
        match self {
            Self::Client(opts) => opts.acl.clone(),
            Self::Server(opts) => opts.acl.clone(),
        }
    }

    pub fn dns_server(&self) -> Address {
        match self {
            Self::Client(opts) => opts.dns_server.clone(),
            Self::Server(opts) => opts.dns_server.clone(),
        }
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

    pub fn bind(&self) -> Address {
        match self {
            Self::Client(opts) => opts.bind.clone(),
            Self::Server(opts) => opts.bind.clone(),
        }
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
}
