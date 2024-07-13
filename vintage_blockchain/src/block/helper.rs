use crate::db::Db;
use crate::tx::TxPool;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use vintage_msg::{Block, BlockBody, BlockHash, BlockHeader, BlockHeight, Tx};
use vintage_utils::{current_timestamp, Timestamp};

fn calc_block_hash(
    block_height: BlockHeight,
    timestamp: Timestamp,
    block_body: &BlockBody,
    prev_hash: &BlockHash,
) -> BlockHash {
    let mut hasher = Sha256::new();
    hasher.update(block_height.to_be_bytes());
    hasher.update(timestamp.to_be_bytes());
    for tx in &block_body.txs {
        hasher.update(tx.id.to_be_bytes());
        hasher.update(&tx.content);
    }
    hasher.update(prev_hash);
    hasher.finalize().into()
}

pub(super) fn check_block_hash(block: &Block, prev_hash: &BlockHash) -> anyhow::Result<()> {
    let hash = calc_block_hash(
        block.header.height,
        block.header.timestamp,
        &block.body,
        prev_hash,
    );
    if block.header.hash == hash {
        Ok(())
    } else {
        Err(anyhow!("block hash is invalid"))
    }
}

pub(super) fn new_block(block_height: BlockHeight, txs: Vec<Tx>, prev_hash: &BlockHash) -> Block {
    let timestamp = current_timestamp();
    let block_body = BlockBody { txs };
    let block_hash = calc_block_hash(block_height, timestamp, &block_body, &prev_hash);
    Block {
        header: BlockHeader {
            height: block_height,
            hash: block_hash,
            timestamp,
        },
        body: block_body,
    }
}

pub(super) fn persist_block(db: &Db, block: &Block) -> anyhow::Result<()> {
    let db_write = db.begin_write()?;
    db_write.write_block(&block)?;
    db_write.commit()?;
    log::info!(
        "block {} persisted, tx count: {}",
        block.header.height,
        block.body.txs.len()
    );
    Ok(())
}

pub(super) fn remove_txs_of_persisted_block(tx_pool: &mut TxPool, block: &Block) {
    for tx in &block.body.txs {
        tx_pool.remove(&tx.id);
    }
}
