mod chain;
mod db;
mod tx;

pub use self::chain::*;
pub use self::tx::*;

use crate::db::BlockChainDb;
use std::sync::Arc;
use vintage_msg::BlockChainMsgChannels;

const BLOCKCHAIN_DB_PATH: &str = "blockchain.db";
const ACT_POOL_CAPACITY: usize = 2000;
const ENTITY_POOP_CAPACITY: usize = 2000;
const MAX_ACT_COUNT_PER_BLOCK: usize = 8000;

impl BlockChain {
    pub async fn create(
        channels: BlockChainMsgChannels,
        db_path: String,
    ) -> anyhow::Result<(BlockChain, TxService)> {
        let db_path = if db_path.is_empty() {
            BLOCKCHAIN_DB_PATH.to_string()
        } else {
            db_path
        };
        let db = BlockChainDb::create(db_path).await?;
        let tx_pool = Arc::new(TxPool::new(ACT_POOL_CAPACITY, ENTITY_POOP_CAPACITY));
        let chain = BlockChain::new(db.clone(), tx_pool.clone(), channels.worker_msg_sender);
        let tx_service = TxService::new(
            db,
            tx_pool,
            channels.msg_receiver,
            channels.network_msg_sender,
        );
        Ok((chain, tx_service))
    }
}
