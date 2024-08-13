use crate::chain::{BlockState, GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT, GENESIS_BLOCK_TIMESTAMP};
use crate::db::{BlockChainDbInner, BlockInDb};
use crate::tx::TxId;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_msg::{Block, BlockHash, BlockHeight, EntityHash, EntityId, Model, UpdateEntityTx};

#[derive(Clone)]
pub(crate) struct BlockChainDb {
    db: Arc<BlockChainDbInner>,
}

// create
pub(crate) async fn create_db_inner(
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
                state: BlockState { total_acts: 0 },
                act_ids: Vec::new(),
                ue_tx_ids: Vec::new(),
            })
        } else {
            let db = self.db.clone();
            spawn_blocking(move || db.get_block(height)).await?
        }
    }

    pub async fn check_act_not_exists(&self, act_id: TxId) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_act_not_exists(&act_id)).await?
    }

    pub async fn check_acts_not_exist(&self, ids: Vec<TxId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_acts_not_exist(&ids)).await?
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
        act_ids: Vec<TxId>,
        ue_tx_ids: Vec<TxId>,
        block: Block,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.commit_block(height, hash, state, act_ids, ue_tx_ids, &block))
            .await?
    }
}
