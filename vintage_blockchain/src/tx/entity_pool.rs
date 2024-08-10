use std::collections::HashMap;
use vintage_msg::{ReqId, UpdateEntities};

pub(crate) struct EntityPool {
    map: HashMap<ReqId, UpdateEntities>,
}

impl EntityPool {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }
}
