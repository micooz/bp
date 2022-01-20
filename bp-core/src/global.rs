use std::{collections::HashMap, sync::Arc};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use quinn::{ClientConfig, ServerConfig};
use tokio::sync::RwLock;
use trust_dns_resolver::TokioAsyncResolver;

use crate::{acl::AccessControlList, net::connection::ConnectionSnapshot, QuinnClientConfig, QuinnServerConfig};

lazy_static! {
    pub static ref GLOBAL_DATA: Arc<GlobalData> = Arc::new(GlobalData::default());
}

#[derive(Default)]
pub struct GlobalData {
    acl: AccessControlList,
    connection_snapshots: Mutex<HashMap<usize, ConnectionSnapshot>>,
    dns_resolver: Arc<RwLock<Option<TokioAsyncResolver>>>,
    quinn_server_config: Mutex<QuinnServerConfig>,
    quinn_client_config: Mutex<QuinnClientConfig>,
}

impl GlobalData {
    pub fn get_acl(&self) -> &AccessControlList {
        &self.acl
    }

    pub fn get_connection_snapshots(&self) -> &Mutex<HashMap<usize, ConnectionSnapshot>> {
        &self.connection_snapshots
    }

    pub async fn remove_connection_snapshot(&self, id: usize) {
        let mut snapshots = self.connection_snapshots.lock();
        snapshots.remove(&id);
    }

    pub async fn set_dns_resolver(&self, resolver: TokioAsyncResolver) {
        let mut inner = self.dns_resolver.write().await;
        *inner = Some(resolver);
    }

    pub fn get_dns_resolver(&self) -> Arc<RwLock<Option<TokioAsyncResolver>>> {
        self.dns_resolver.clone()
    }

    pub fn set_quinn_server_config(&self, config: ServerConfig) {
        let mut server_config = self.quinn_server_config.lock();
        *server_config = QuinnServerConfig::new(config);
    }

    pub fn get_quinn_server_config(&self) -> ServerConfig {
        let server_config = self.quinn_server_config.lock();
        server_config.inner()
    }

    pub fn set_quinn_client_config(&self, config: ClientConfig) {
        let mut inner = self.quinn_client_config.lock();
        *inner = QuinnClientConfig::new(config);
    }

    pub fn get_quinn_client_config(&self) -> ClientConfig {
        let client_config = self.quinn_client_config.lock();
        client_config.inner()
    }
}
