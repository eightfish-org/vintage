use crate::db::AsyncBlockChainDb;
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use vintage_msg::{Block, BlockBody, BlockHash, BlockHeader, BlockHeight, Tx};
use vintage_utils::{current_timestamp, Bytes, Timestamp};

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
    hasher.update(prev_hash.0);
    Bytes(hasher.finalize().into())
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

pub(super) async fn persist_block(db: &AsyncBlockChainDb, block: Block) -> anyhow::Result<()> {
    let block_height = block.header.height;
    let tx_count = block.body.txs.len();
    db.write_block(block).await?;
    log::info!("block {} persisted, tx count: {}", block_height, tx_count);
    Ok(())
}
