use crate::acl::AccessControlList;
use crate::net::connection::ConnectionSnapshot;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct SharedData {
    connection_snapshots: Mutex<HashMap<usize, ConnectionSnapshot>>,

    acl: AccessControlList,
}

impl SharedData {
    pub fn get_connection_snapshots(&self) -> &Mutex<HashMap<usize, ConnectionSnapshot>> {
        &self.connection_snapshots
    }

    pub fn get_acl(&self) -> &AccessControlList {
        &self.acl
    }

    pub async fn remove_connection_snapshot(&self, id: usize) {
        let mut snapshots = self.connection_snapshots.lock().await;
        snapshots.remove(&id);
    }
}
