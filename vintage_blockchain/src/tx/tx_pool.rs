use crate::ActPool;
use std::sync::{Mutex, MutexGuard};
use vintage_msg::Hashed;

pub(crate) type TxId = Hashed;

pub(crate) struct TxPool {
    inner: Mutex<ActPool>,
}

impl TxPool {
    pub fn new(act_capacity: usize) -> Self {
        Self {
            inner: Mutex::new(ActPool::with_capacity(act_capacity)),
        }
    }

    pub fn guard(&self) -> MutexGuard<'_, ActPool> {
        self.inner.lock().unwrap()
    }
}
