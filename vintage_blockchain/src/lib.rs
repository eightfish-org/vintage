mod api;
mod chain;
mod consensus;
mod db;
mod network;
mod proxy;
mod service;
mod tx;
mod wasm_db;

pub use self::api::*;
pub(crate) use self::chain::*;
pub use self::consensus::*;
pub(crate) use self::db::*;
pub(crate) use self::network::*;
pub(crate) use self::proxy::*;
pub use self::service::*;
pub(crate) use self::tx::*;
pub(crate) use self::wasm_db::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vintage_msg::BlockChainMsgChannels;
use vintage_network::client::NetworkClient;
use vintage_utils::ServiceStarter;

const ACT_POOL_CAPACITY: usize = 1000;
const WASM_POOL_CAPACITY: usize = 4;
const MAX_ACT_COUNT_PER_BLOCK: usize = 4000;
const MAX_UE_TX_COUNT_PER_BLOCK: usize = 4000;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockChainConfig {
    pub db_path: String,
    pub wasm_db_path: String,
}

pub enum BlockChain {}

impl BlockChain {
    pub async fn create(
        config: BlockChainConfig,
        block_interval: u64,
        active_number_of_nodes: usize,
        channels: BlockChainMsgChannels,
        client: NetworkClient,
    ) -> anyhow::Result<(
        BlockConsensusImpl,
        BlockChainApiImpl,
        ServiceStarter<BlockChainService>,
        ServiceStarter<BlockSyncService>,
    )> {
        let blockchain_db = BlockChainDb::new(create_blockchain_db_inner(config.db_path).await?);
        let wasm_db = WasmDb::new(create_wasm_db_inner(config.wasm_db_path).await?);
        let tx_pool = Arc::new(TxPool::new(ACT_POOL_CAPACITY, WASM_POOL_CAPACITY));
        let network_msg_sender = MsgToNetworkSender::new(channels.network_msg_sender);
        let proxy_msg_sender = MsgToProxySender::new(channels.proxy_msg_sender);
        let client =
            BlockChainNetworkClient::new(NetworkClientWrapper::new(client, active_number_of_nodes));

        let blockchain_core = Arc::new(tokio::sync::Mutex::new(BlockChainCore::new(
            blockchain_db.clone(),
            tx_pool.clone(),
            proxy_msg_sender,
        )));
        let block_sync_service =
            BlockSyncService::new(block_interval, client, channels.block_synced_sender);
        let blockchain_service = BlockChainService::new(
            blockchain_db.clone(),
            wasm_db,
            tx_pool,
            channels.msg_receiver,
            network_msg_sender,
        );

        Ok((
            BlockConsensusImpl::new(blockchain_core.clone()),
            BlockChainApiImpl::new(blockchain_db),
            ServiceStarter::new(blockchain_service),
            ServiceStarter::new_with_input(block_sync_service, blockchain_core),
        ))
    }
}
