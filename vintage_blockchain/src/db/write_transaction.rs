use crate::db::LastBlockHeight;
use crate::db::Txs;
use crate::db::{BlockInDb, Blocks};
use redb::{CommitError, WriteTransaction};
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

    // complete all operations within a single transaction
    pub fn write_block(&self, block: &Block) -> anyhow::Result<()> {
        // update last_block_height
        {
            let mut table_lbh = LastBlockHeight::open_writable_table(&self.transaction)?;
            table_lbh.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table_txs = Txs::open_writable_table(&self.transaction)?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table_txs.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table_blocks = Blocks::open_writable_table(&self.transaction)?;
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
