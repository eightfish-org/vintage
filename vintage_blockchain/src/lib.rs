mod api;
mod chain;
mod consensus;
mod db;
mod network;
mod proxy;
mod service;
mod tx;

pub use self::api::*;
pub(crate) use self::chain::*;
pub use self::consensus::*;
pub(crate) use self::db::*;
pub(crate) use self::network::*;
pub(crate) use self::proxy::*;
pub use self::service::*;
pub(crate) use self::tx::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vintage_msg::BlockChainMsgChannels;
use vintage_utils::ServiceStarter;

const ACT_POOL_CAPACITY: usize = 1000;
const MAX_ACT_COUNT_PER_BLOCK: usize = 4000;
const MAX_UE_TX_COUNT_PER_BLOCK: usize = 4000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockChainConfig {
    pub db_path: String,
}

pub enum BlockChain {}

impl BlockChain {
    pub async fn create(
        config: BlockChainConfig,
        channels: BlockChainMsgChannels,
    ) -> anyhow::Result<(
        BlockConsensusImpl,
        BlockChainApiImpl,
        ServiceStarter<BlockChainService>,
        ServiceStarter<BlockSyncService>,
    )> {
        let db_inner = create_db_inner(config.db_path).await?;
        let db = BlockChainDb::new(db_inner.clone());
        let tx_pool = Arc::new(TxPool::new(ACT_POOL_CAPACITY));

        let blockchain_core = Arc::new(tokio::sync::Mutex::new(BlockChainCore::new(
            db.clone(),
            tx_pool.clone(),
            MsgToProxySender::new(channels.proxy_msg_sender),
        )));

        let block_consensus = BlockConsensusImpl::new(blockchain_core);
        let blockchain_api = BlockChainApiImpl::new(db.clone());
        let blockchain_service = ServiceStarter::new(BlockChainService::new(
            db,
            tx_pool,
            channels.msg_receiver,
            MsgToNetworkSender::new(channels.network_msg_sender),
        ));
        let block_sync_service = ServiceStarter::new(BlockSyncService::new());

        Ok((
            block_consensus,
            blockchain_api,
            blockchain_service,
            block_sync_service,
        ))
    }
}
