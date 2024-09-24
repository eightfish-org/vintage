use crate::chain::{BlockState, GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT, GENESIS_BLOCK_TIMESTAMP};
use crate::db::{BlockChainDbInner, BlockInDb};
use crate::tx::TxId;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_msg::{
    Block, BlockHash, BlockHeight, EntityHash, EntityId, Model, UpdateEntityTx, WasmId, WasmInfo,
};

#[derive(Clone)]
pub(crate) struct BlockChainDb {
    db: Arc<BlockChainDbInner>,
}

// create
pub(crate) async fn create_blockchain_db_inner(
    path: impl AsRef<Path> + Send + 'static,
) -> anyhow::Result<Arc<BlockChainDbInner>> {
    let db = spawn_blocking(|| BlockChainDbInner::create(path)).await??;
    Ok(Arc::new(db))
}

impl BlockChainDb {
    pub(crate) fn new(db: Arc<BlockChainDbInner>) -> Self {
        Self { db }
    }
}

// read
impl BlockChainDb {
    pub async fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_block_height()).await?
    }

    pub async fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        if height == GENESIS_BLOCK_HEIGHT {
            Ok(BlockInDb {
                hash: GENESIS_BLOCK_HASH,
                timestamp: GENESIS_BLOCK_TIMESTAMP,
                state: BlockState { total_act_txs: 0 },
                act_tx_ids: Default::default(),
                ue_tx_ids: Default::default(),
                wasm_ids: Default::default(),
            })
        } else {
            let db = self.db.clone();
            spawn_blocking(move || db.get_block(height)).await?
        }
    }

    pub async fn get_network_block(&self, height: BlockHeight) -> anyhow::Result<Block> {
        if height == GENESIS_BLOCK_HEIGHT {
            Ok(Block {
                timestamp: GENESIS_BLOCK_TIMESTAMP,
                act_txs: Default::default(),
                ue_txs: Default::default(),
                wasm_txs: Default::default(),
            })
        } else {
            let db = self.db.clone();
            spawn_blocking(move || db.get_network_block(height)).await?
        }
    }

    pub async fn check_act_not_exists(&self, act_tx_id: TxId) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_act_tx_not_exists(&act_tx_id)).await?
    }

    pub async fn check_act_txs_not_exist(&self, ids: Vec<TxId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_act_txs_not_exist(&ids)).await?
    }

    pub async fn check_ue_txs_not_exist(&self, tx_ids: Vec<TxId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_ue_txs_not_exist(&tx_ids)).await?
    }

    pub async fn ue_tx_exists_in_pool(&self, tx_id: TxId) -> anyhow::Result<bool> {
        let db = self.db.clone();
        spawn_blocking(move || db.ue_tx_exists_in_pool(&tx_id)).await?
    }

    pub async fn get_ue_txs_in_pool(
        &self,
        count: usize,
    ) -> anyhow::Result<(Vec<TxId>, Vec<UpdateEntityTx>)> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_ue_txs_in_pool(count)).await?
    }

    pub async fn get_entity(
        &self,
        model: Model,
        entity_id: EntityId,
    ) -> anyhow::Result<EntityHash> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_entity(&model, &entity_id)).await?
    }

    pub async fn check_wasm_tx_not_exists(&self, wasm_id: WasmId) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_wasm_tx_not_exists(&wasm_id)).await?
    }

    pub async fn check_wasm_txs_not_exist(&self, wasm_id: Vec<WasmId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_wasm_txs_not_exist(&wasm_id)).await?
    }

    pub async fn get_upgrade_wasm_ids(
        &self,
        block_height: BlockHeight,
    ) -> anyhow::Result<Vec<WasmId>> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_upgrade_wasm_ids(block_height)).await?
    }

    pub async fn _get_wasm_tx(&self, wasm_id: WasmId) -> anyhow::Result<WasmInfo> {
        let db = self.db.clone();
        spawn_blocking(move || db._get_wasm_tx(&wasm_id)).await?
    }
}

// write
impl BlockChainDb {
    pub async fn insert_ue_tx_to_pool(
        &self,
        tx_id: TxId,
        tx: UpdateEntityTx,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.insert_ue_tx_to_pool(&tx_id, &tx)).await?
    }

    pub async fn commit_block(
        &self,
        height: BlockHeight,
        hash: BlockHash,
        state: BlockState,
        act_tx_ids: Vec<TxId>,
        ue_tx_ids: Vec<TxId>,
        wasm_ids: Vec<WasmId>,
        block: Block,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || {
            db.commit_block(height, hash, state, act_tx_ids, ue_tx_ids, wasm_ids, block)
        })
        .await?
    }
}
