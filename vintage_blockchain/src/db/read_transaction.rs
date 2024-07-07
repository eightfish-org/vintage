use crate::db::LastBlockHeight;
use crate::db::Txs;
use crate::db::{BlockInDb, Blocks};
use crate::genesis::{GENESIS_BLOCK_HASH, GENESIS_BLOCK_HEIGHT};
use anyhow::anyhow;
use redb::ReadTransaction;
use vintage_msg::{BlockHash, BlockHeight, Tx, TxId};

pub(crate) struct DbRead<'db> {
    transaction: ReadTransaction<'db>,
}

impl<'db> DbRead<'db> {
    pub fn new(transaction: ReadTransaction<'db>) -> Self {
        Self { transaction }
    }

    pub fn check_tx_not_exists(&self, id: TxId) -> anyhow::Result<()> {
        let table_txs = Txs::open_table(&self.transaction)?;
        table_txs.check_tx_not_exists(id)
    }

    pub fn check_all_txs_not_exist(&self, txs: &Vec<Tx>) -> anyhow::Result<()> {
        let table_txs = Txs::open_table(&self.transaction)?;
        table_txs.check_all_txs_not_exist(txs)
    }

    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let table_lbh = LastBlockHeight::open_table(&self.transaction)?;
        table_lbh.get_last_block_height()
    }

    pub fn get_block(&self, block_height: BlockHeight) -> anyhow::Result<Option<BlockInDb>> {
        let table_blocks = Blocks::open_table(&self.transaction)?;
        table_blocks.get_block(block_height)
    }
}

impl<'db> DbRead<'db> {
    pub fn get_block_hash(&self, block_height: BlockHeight) -> anyhow::Result<BlockHash> {
        let hash = if block_height == GENESIS_BLOCK_HEIGHT {
            GENESIS_BLOCK_HASH
        } else {
            self.get_block(block_height)?
                .ok_or_else(|| anyhow!(" block {} not found", block_height))?
                .hash
        };
        Ok(hash)
    }
}
