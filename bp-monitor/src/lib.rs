use bp_lib::Connection;
use std::ops::Deref;
use std::sync::Arc;
use std::{collections::HashSet, hash::Hash};
use tokio::sync::RwLock;

mod cmd;
mod context;
mod net;

pub use cmd::{Command, MonitorCommand};
pub use net::handle_conn;

// ConnectionRecord
type ArcRwLockConnection = Arc<RwLock<Connection>>;

pub struct ConnectionRecord {
    id: usize,
    inner: ArcRwLockConnection,
}

impl ConnectionRecord {
    pub fn new(id: usize, conn: ArcRwLockConnection) -> Self {
        Self { id, inner: conn }
    }
}

impl Clone for ConnectionRecord {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: self.inner.clone(),
        }
    }
}

impl Deref for ConnectionRecord {
    type Target = ArcRwLockConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Hash for ConnectionRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.finish();
    }
}

impl PartialEq for ConnectionRecord {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ConnectionRecord {}

// SharedContext

#[derive(Clone)]
pub struct SharedContext {
    pub shared_conns: Arc<RwLock<HashSet<ConnectionRecord>>>,
}
