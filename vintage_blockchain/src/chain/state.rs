use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct BlockState {
    pub total_acts: u64, // 全区块链的act总数，不是当前块的act数量
}