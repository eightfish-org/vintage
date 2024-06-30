use crate::table_blocks::{db_insert_block, BlockInDb};
use crate::{table_blocks, table_last_block_height, table_txs};
use redb::{CommitError, Database, TransactionError, WriteTransaction};
use vintage_msg::{Block, TxId};

pub(crate) struct BlockChainDbWriter<'db> {
    transaction: WriteTransaction<'db>,
}

impl<'db> BlockChainDbWriter<'db> {
    pub fn begin(database: &'db Database) -> Result<Self, TransactionError> {
        let transaction = database.begin_write()?;
        Ok(Self { transaction })
    }

    pub fn commit(self) -> Result<(), CommitError> {
        self.transaction.commit()
    }

    // complete all operations within a single transaction
    pub fn persist_block(&self, block: &Block) -> anyhow::Result<()> {
        // update last_block_height
        {
            let mut table = table_last_block_height::open_table(&self.transaction)?;
            table.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table = table_txs::open_table(&self.transaction)?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table = table_blocks::open_table(&self.transaction)?;
            db_insert_block(
                &mut table,
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
