use crate::block::calc_block_hash;
use crate::db::{blocks, last_block_height, txs, DB};
use anyhow::anyhow;
use log::error;
use redb::Database;
use vintage_msg::{Block, BlockHeight};
use vintage_utils::Pool;

pub type BlockPool = Pool<BlockHeight, Block>;

pub fn block_handler(db: &Database, block_pool: &mut BlockPool, block: Block) {
    if let Err(e) = block_handler_impl(db, block_pool, block) {
        error!("block handle error: {}", e);
    }
}

fn block_handler_impl(
    db: &Database,
    block_pool: &mut BlockPool,
    block: Block,
) -> anyhow::Result<()> {
    {
        // ReadTransaction
        let transaction = db.begin_read()?;

        // check txs not exist
        {
            let table_txs = txs::open_table(&transaction)?;
            for tx in &block.body.txs {
                DB::check_tx_not_exists(&table_txs, tx.id)?;
            }
        }

        // read last_block_height
        let last_block_height = {
            let table_lbh = last_block_height::open_table(&transaction)?;
            DB::get_last_block_height(&table_lbh)?
        };

        // check block height
        let next_block_height = last_block_height + 1;
        if block.header.height > next_block_height {
            block_pool.insert(block);
            return Ok(());
        } else if block.header.height < next_block_height {
            return Err(anyhow!(
                "the block height {}, less than {}",
                block.header.height,
                next_block_height,
            ));
        }

        // read last_block
        let last_block = {
            let table_blocks = blocks::open_table(&transaction)?;
            DB::get_block(&table_blocks, last_block_height)?
                .ok_or_else(|| anyhow!("last block not found"))?
        };

        // check block hash
        let hash = calc_block_hash(
            block.header.height,
            block.header.timestamp,
            &block.body,
            &last_block.hash,
        );
        if hash != block.header.hash {
            return Err(anyhow!("block hash is invalid"));
        }
    }

    {
        // WriteTransaction
        let transaction = db.begin_write()?;

        // write block
        DB::write_block(&transaction, &block)?;
    }

    Ok(())
}
