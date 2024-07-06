use crate::db::blocks::BlockInDb;
use crate::db::{blocks, last_block_height, txs, DB};
use redb::WriteTransaction;
use vintage_msg::{Block, TxId};

impl DB {
    // complete all operations within a single transaction
    pub fn write_block<'db>(
        transaction: &WriteTransaction<'db>,
        block: &Block,
    ) -> anyhow::Result<()> {
        // update last_block_height
        {
            let mut table = last_block_height::open_writable_table(&transaction)?;
            table.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table = txs::open_writable_table(&transaction)?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table = blocks::open_writable_table(&transaction)?;
            DB::insert_block(
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
