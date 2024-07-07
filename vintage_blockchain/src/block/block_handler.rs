use crate::block::BlockPool;
use crate::block::{check_block_hash, check_block_height};
use crate::db::{Db, DbWrite};
use anyhow::anyhow;
use log::error;
use vintage_msg::Block;

pub(crate) fn block_handler(db: &Db, block_pool: &mut BlockPool, block: Block) {
    if let Err(e) = block_handler_impl(db, block_pool, block) {
        error!("block handle error: {}", e);
    }
}

fn block_handler_impl(db: &Db, block_pool: &mut BlockPool, block: Block) -> anyhow::Result<()> {
    {
        // ReadTransaction
        let transaction = db.begin_read()?;

        // check all txs not exist in db
        transaction.check_all_txs_not_exist(&block.body.txs)?;

        // get last_block height
        let last_block_height = transaction.get_last_block_height()?;

        // check block height
        if !check_block_height(block.header.height, last_block_height)? {
            let block_height = block.header.height;
            block_pool.insert(block);
            return Err(anyhow!(
                "block height {}, last block height {}",
                block_height,
                last_block_height
            ));
        }

        // read last_block hash
        let last_block_hash = transaction.get_block_hash(last_block_height)?;

        // check block hash
        check_block_hash(&block, &last_block_hash)?;
    }

    {
        // WriteTransaction
        let transaction = db.begin_write()?;

        // write block
        DbWrite::write_block(&transaction, &block)?;
    }

    Ok(())
}
