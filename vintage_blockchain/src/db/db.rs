use crate::chain::{BlockState, GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT, GENESIS_BLOCK_TIMESTAMP};
use crate::db::{BlockChainDbInner, BlockInDb};
use crate::tx::ActId;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_msg::{Block, BlockHash, BlockHeight};

#[derive(Clone)]
pub(crate) struct BlockChainDb {
    db: Arc<BlockChainDbInner>,
}

// create
impl BlockChainDb {
    pub async fn create(path: impl AsRef<Path> + Send + 'static) -> anyhow::Result<Self> {
        let db = spawn_blocking(|| BlockChainDbInner::create(path)).await??;
        Ok(Self { db: Arc::new(db) })
    }
}

// read
impl BlockChainDb {
    pub async fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_last_block_height()).await?
    }

    pub async fn check_act_not_exists(&self, act_id: ActId) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_act_not_exists(&act_id)).await?
    }

    pub async fn check_acts_not_exist(&self, ids: Vec<ActId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_acts_not_exist(&ids)).await?
    }

    pub async fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        if height == GENESIS_BLOCK_HEIGHT {
            Ok(BlockInDb {
                hash: GENESIS_BLOCK_HASH,
                timestamp: GENESIS_BLOCK_TIMESTAMP,
                state: BlockState { total_acts: 0 },
                act_ids: Vec::new(),
            })
        } else {
            let db = self.db.clone();
            spawn_blocking(move || db.get_block(height)).await?
        }
    }
}

// write
impl BlockChainDb {
    pub async fn commit_block(
        &self,
        height: BlockHeight,
        hash: BlockHash,
        state: BlockState,
        act_ids: Vec<ActId>,
        block: Block,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.commit_block(height, hash, state, act_ids, &block)).await?
    }
}
