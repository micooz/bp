use crate::acl::AccessControlList;
use crate::net::connection::ConnectionSnapshot;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use trust_dns_resolver::TokioAsyncResolver;

lazy_static! {
    pub static ref SHARED_DATA: Arc<SharedData> = Arc::new(SharedData::default());
}

#[derive(Default)]
pub struct SharedData {
    connection_snapshots: Mutex<HashMap<usize, ConnectionSnapshot>>,

    acl: AccessControlList,

    dns_resolver: Arc<RwLock<Option<TokioAsyncResolver>>>,
}

impl SharedData {
    pub fn get_connection_snapshots(&self) -> &Mutex<HashMap<usize, ConnectionSnapshot>> {
        &self.connection_snapshots
    }

    pub fn get_acl(&self) -> &AccessControlList {
        &self.acl
    }

    pub async fn set_dns_resolver(&self, resolver: TokioAsyncResolver) {
        let mut inner = self.dns_resolver.write().await;
        *inner = Some(resolver);
    }

    pub fn get_dns_resolver(&self) -> Arc<RwLock<Option<TokioAsyncResolver>>> {
        self.dns_resolver.clone()
    }

    pub async fn remove_connection_snapshot(&self, id: usize) {
        let mut snapshots = self.connection_snapshots.lock().await;
        snapshots.remove(&id);
    }
}
