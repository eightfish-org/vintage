use crate::block::check_block_hash;
use crate::block::helper::{new_block, persist_block, remove_txs_of_persisted_block};
use crate::block::{BlockMsg, BlockMsgPool};
use crate::db::Db;
use crate::tx::TxPool;
use crate::MAX_TXS_PER_BLOCK;
use anyhow::anyhow;
use tokio::sync::mpsc;
use vintage_msg::{Block, BlockProduction, NetworkMsg};
use vintage_utils::{SendMsg, WithId};

pub(crate) fn block_msg_handler(
    db: &Db,
    tx_pool: &mut TxPool,
    block_msg_pool: &mut BlockMsgPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    block_msg: BlockMsg,
) -> anyhow::Result<()> {
    let mut next_block_height = {
        db.begin_read()?
            .open_last_block_height()?
            .get_last_block_height()?
    } + 1;
    let block_height = block_msg.id().clone();

    // check block_height
    if block_height < next_block_height {
        return Err(anyhow!(
            "block_height {} < next_block_height {}",
            block_height,
            next_block_height,
        ));
    } else if block_height > next_block_height {
        log::info!(
            "block_height {} > next_block_height {}",
            block_height,
            next_block_height
        );
        block_msg_pool.insert(block_msg);
        return Ok(());
    }

    block_msg_handler_impl(db, tx_pool, network_msg_sender, block_msg)?;
    next_block_height += 1;

    loop {
        match block_msg_pool.remove(&next_block_height) {
            Some(msg) => {
                block_msg_handler_impl(db, tx_pool, network_msg_sender, msg)?;
                next_block_height += 1;
            }
            None => break,
        }
    }
    Ok(())
}

fn block_msg_handler_impl(
    db: &Db,
    tx_pool: &mut TxPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    msg: BlockMsg,
) -> anyhow::Result<()> {
    match msg {
        BlockMsg::Block(block) => block_handler(db, tx_pool, block),
        BlockMsg::BlockProduction(block_production) => {
            block_production_handler(db, tx_pool, network_msg_sender, block_production)
        }
    }
}

fn block_handler(db: &Db, tx_pool: &mut TxPool, block: Block) -> anyhow::Result<()> {
    let prev_block_hash = {
        // DbRead
        let db_read = db.begin_read()?;

        let table_txs = db_read.open_txs()?;
        for tx in &block.body.txs {
            table_txs.check_tx_not_exists(tx.id)?;
        }

        db_read.get_block_hash(block.header.height - 1)?
    };

    check_block_hash(&block, &prev_block_hash)?;

    persist_block(db, &block)?;
    remove_txs_of_persisted_block(tx_pool, &block);

    Ok(())
}

fn block_production_handler(
    db: &Db,
    tx_pool: &mut TxPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    block_production: BlockProduction,
) -> anyhow::Result<()> {
    let prev_block_hash = {
        db.begin_read()?
            .get_block_hash(block_production.block_height - 1)?
    };

    let block = new_block(
        block_production.block_height,
        tx_pool.get_values(MAX_TXS_PER_BLOCK),
        &prev_block_hash,
    );

    persist_block(db, &block)?;
    remove_txs_of_persisted_block(tx_pool, &block);

    network_msg_sender.send_msg(NetworkMsg::BroadcastBlock(block));
    Ok(())
}
