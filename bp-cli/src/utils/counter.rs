use std::fmt::{Display, Formatter, Result};

#[derive(Default)]
pub struct Counter {
    inner: usize,
}

impl Counter {
    pub fn inc(&mut self) {
        self.inner += 1;
    }
    pub fn dec(&mut self) {
        self.inner -= 1;
    }
}

impl Display for Counter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.inner)
    }
}
