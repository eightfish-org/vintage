use crate::block::genesis::{GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT};
use crate::db::BlockChainDb;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use vintage_msg::{Block, BlockHash, BlockHeight, TxId};

#[derive(Clone)]
pub struct AsyncBlockChainDb {
    pub db: Arc<Mutex<BlockChainDb>>,
}

impl AsyncBlockChainDb {
    pub async fn create(path: impl AsRef<Path> + Send + 'static) -> anyhow::Result<Self> {
        let db = spawn_blocking(|| BlockChainDb::create(path)).await??;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

impl AsyncBlockChainDb {
    pub async fn check_tx_not_exists(&self, id: TxId) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().check_tx_not_exists(id)).await?
    }

    pub async fn check_txs_not_exist(&self, ids: Vec<TxId>) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().check_txs_not_exist(&ids)).await?
    }

    pub async fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().get_last_block_height()).await?
    }

    pub async fn get_block_hash(&self, block_height: BlockHeight) -> anyhow::Result<BlockHash> {
        if block_height == GENESIS_BLOCK_HEIGHT {
            Ok(GENESIS_BLOCK_HASH)
        } else {
            let db = self.db.clone();
            spawn_blocking(move || db.lock().unwrap().get_block_hash(block_height)).await?
        }
    }

    pub async fn write_block(&self, block: Block) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().write_block(&block)).await?
    }
}
