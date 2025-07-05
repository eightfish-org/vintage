use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct BlockState {
    pub total_act_txs: u64, // 全区块链的act总数，不是当前块的act数量
}
