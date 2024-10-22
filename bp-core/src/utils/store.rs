use std::collections::LinkedList;

use bytes::Bytes;

#[derive(Default, Debug)]
pub struct Store {
    inner: LinkedList<Bytes>,
}

impl Store {
    pub fn push_front(&mut self, data: Bytes) {
        self.inner.push_front(data);
    }

    pub fn push_back(&mut self, data: Bytes) {
        self.inner.push_back(data);
    }

    pub fn pop_back(&mut self) -> Option<Bytes> {
        self.inner.pop_back()
    }

    pub fn pull(&mut self, n: usize) -> Bytes {
        if n == 0 || self.inner.is_empty() {
            return Bytes::default();
        }

        let mut arr = Vec::with_capacity(self.inner.len());
        let mut total = 0;
        let mut consumed_item_count = 0;

        for item in self.inner.iter_mut() {
            total += item.len();
            if total > n {
                let mid = item.len() - (total - n);
                let extra = item.split_to(mid);
                arr.push(extra);
                break;
            } else {
                arr.push(item.clone());
                consumed_item_count += 1;
            }
        }

        self.inner = self.inner.split_off(consumed_item_count);

        Bytes::from_iter(arr.concat().into_iter())
    }

    pub fn pull_all(&mut self) -> Bytes {
        let mut arr = vec![];
        while let Some(item) = self.inner.pop_front() {
            arr.push(item);
        }
        Bytes::from_iter(arr.concat().into_iter())
    }

    pub fn len(&self) -> usize {
        self.inner.iter().fold(0, |acc, next| acc + next.len())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}
