use std::collections::HashMap;
use vintage_msg::{Block, BlockHeight};

pub(crate) struct BlockPool {
    map: HashMap<BlockHeight, Block>,
}

impl BlockPool {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
