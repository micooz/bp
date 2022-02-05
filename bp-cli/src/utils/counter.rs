use std::{
    fmt::{Display, Formatter, Result},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
pub struct Counter {
    inner: AtomicUsize,
}

impl Counter {
    pub fn inc(&self) {
        self.inner.fetch_add(1, Ordering::Relaxed);
    }
    pub fn dec(&self) {
        self.inner.fetch_sub(1, Ordering::Relaxed);
    }
    pub fn value(&self) -> usize {
        self.inner.load(Ordering::Relaxed)
    }
}

impl Display for Counter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}
