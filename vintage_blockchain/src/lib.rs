mod api;
mod chain;
mod db;
mod network;
mod service;
mod tx;

pub use self::api::*;
pub use self::chain::*;
pub(crate) use self::db::*;
pub(crate) use self::network::*;
pub use self::service::*;
pub(crate) use self::tx::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vintage_msg::BlockChainMsgChannels;

const ACT_POOL_CAPACITY: usize = 1000;
const MAX_ACT_COUNT_PER_BLOCK: usize = 4000;
const MAX_UE_TX_COUNT_PER_BLOCK: usize = 4000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockChainConfig {
    pub db_path: String,
}

impl BlockChain {
    pub async fn create(
        config: BlockChainConfig,
        channels: BlockChainMsgChannels,
    ) -> anyhow::Result<(BlockChain, BlockChainApiImpl, BlockChainService)> {
        let db_inner = create_db_inner(config.db_path).await?;
        let db = BlockChainDb::new(db_inner.clone());
        let tx_pool = Arc::new(TxPool::new(ACT_POOL_CAPACITY));
        let network_msg_sender = MsgToNetworkSender::new(channels.network_msg_sender);

        let chain = BlockChain::new(db.clone(), tx_pool.clone(), channels.proxy_msg_sender);
        let api = BlockChainApiImpl::new(db.clone());
        let tx_service =
            BlockChainService::new(db, tx_pool, channels.msg_receiver, network_msg_sender);

        Ok((chain, api, tx_service))
    }
}
