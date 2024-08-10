use crate::ActPool;
use crate::EntityPool;
use std::sync::{Mutex, MutexGuard};

pub(crate) struct TxPool {
    inner: Mutex<TxPoolInner>,
}

pub(crate) struct TxPoolInner {
    pub acts: ActPool,
    pub entities: EntityPool,
}

impl TxPool {
    pub fn new(act_capacity: usize, entity_capacity: usize) -> Self {
        Self {
            inner: Mutex::new(TxPoolInner {
                acts: ActPool::with_capacity(act_capacity),
                entities: EntityPool::with_capacity(entity_capacity),
            }),
        }
    }

    pub fn guard(&self) -> MutexGuard<'_, TxPoolInner> {
        self.inner.lock().unwrap()
    }
}
