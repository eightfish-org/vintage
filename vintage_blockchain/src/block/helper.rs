use anyhow::anyhow;
use sha2::{Digest, Sha256};
use vintage_msg::{Block, BlockBody, BlockHash, BlockHeight};
use vintage_utils::{Pool, Timestamp};

pub(crate) type BlockPool = Pool<BlockHeight, Block>;

pub(super) fn check_block_height(
    block_height: BlockHeight,
    last_block_height: BlockHeight,
) -> anyhow::Result<bool> {
    let new_block_height = last_block_height + 1;
    if block_height < new_block_height {
        Err(anyhow!(
            "the block height {}, less than {}",
            block_height,
            new_block_height,
        ))
    } else {
        Ok(block_height == new_block_height)
    }
}

pub(super) fn calc_block_hash(
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
