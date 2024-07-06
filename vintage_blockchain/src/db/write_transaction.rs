use crate::db::blocks::BlockInDb;
use crate::db::{blocks, last_block_height, txs, DbTable};
use redb::WriteTransaction;
use vintage_msg::{Block, TxId};

pub(crate) enum DbWrite {}

impl DbWrite {
    // complete all operations within a single transaction
    pub fn write_block<'db>(
        transaction: &WriteTransaction<'db>,
        block: &Block,
    ) -> anyhow::Result<()> {
        // update last_block_height
        {
            let mut table_lbh = last_block_height::open_writable_table(&transaction)?;
            table_lbh.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table_txs = txs::open_writable_table(&transaction)?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table_txs.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table_blocks = blocks::open_writable_table(&transaction)?;
            DbTable::insert_block(
                &mut table_blocks,
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
