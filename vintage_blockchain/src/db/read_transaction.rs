use crate::db::Blocks;
use crate::db::Txs;
use crate::db::{BlocksR, LastBlockHeight, LastBlockHeightR, TxsR};
use crate::genesis::{GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT};
use anyhow::anyhow;
use redb::{ReadTransaction, TableError};
use vintage_msg::{BlockHash, BlockHeight};

pub(crate) struct DbRead<'db> {
    transaction: ReadTransaction<'db>,
}

impl<'db> DbRead<'db> {
    pub fn new(transaction: ReadTransaction<'db>) -> Self {
        Self { transaction }
    }

    pub fn open_last_block_height(&self) -> Result<LastBlockHeightR, TableError> {
        LastBlockHeight::open_table(&self.transaction)
    }

    pub fn open_blocks(&self) -> Result<BlocksR, TableError> {
        Blocks::open_table(&self.transaction)
    }

    pub fn open_txs(&self) -> Result<TxsR, TableError> {
        Txs::open_table(&self.transaction)
    }
}

impl<'db> DbRead<'db> {
    pub fn get_block_hash(&self, block_height: BlockHeight) -> anyhow::Result<BlockHash> {
        let hash = if block_height == GENESIS_BLOCK_HEIGHT {
            GENESIS_BLOCK_HASH
        } else {
            self.open_blocks()?
                .get_block(block_height)?
                .ok_or_else(|| anyhow!(" block {} not found", block_height))?
                .hash
        };
        Ok(hash)
    }
}
