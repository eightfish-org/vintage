use sha2::{Digest, Sha256};
use vintage_msg::{BlockBody, BlockHash, BlockHeight};
use vintage_utils::Timestamp;

pub(crate) fn calc_block_hash(
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
