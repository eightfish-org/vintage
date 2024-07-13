use crate::db::Txs;
use crate::db::{BlockInDb, Blocks};
use crate::db::{BlocksW, LastBlockHeight, LastBlockHeightW, TxsW};
use redb::{CommitError, TableError, WriteTransaction};
use vintage_msg::{Block, TxId};

pub(crate) struct DbWrite<'db> {
    transaction: WriteTransaction<'db>,
}

impl<'db> DbWrite<'db> {
    pub fn new(transaction: WriteTransaction<'db>) -> Self {
        Self { transaction }
    }

    pub fn commit(self) -> Result<(), CommitError> {
        self.transaction.commit()
    }

    pub fn open_last_block_height<'txn>(
        &'txn self,
    ) -> Result<LastBlockHeightW<'db, 'txn>, TableError> {
        LastBlockHeight::open_writable_table(&self.transaction)
    }

    pub fn open_blocks<'txn>(&'txn self) -> Result<BlocksW<'db, 'txn>, TableError> {
        Blocks::open_writable_table(&self.transaction)
    }

    pub fn open_txs<'txn>(&'txn self) -> Result<TxsW<'db, 'txn>, TableError> {
        Txs::open_writable_table(&self.transaction)
    }

    // complete all operations within a single transaction
    pub fn write_block(&self, block: &Block) -> anyhow::Result<()> {
        // update last_block_height
        {
            let mut table_lbh = self.open_last_block_height()?;
            table_lbh.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table_txs = self.open_txs()?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table_txs.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table_blocks = self.open_blocks()?;
            table_blocks.insert_block(
                block.header.height,
                &BlockInDb {
                    hash: block.header.hash,
                    timestamp: block.header.timestamp,
                    tx_ids,
                },
            )?;
        }

        Ok(())
    }
}
