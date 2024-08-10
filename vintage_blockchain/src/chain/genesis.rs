use vintage_msg::{BlockHash, BlockHeight};
use vintage_utils::Timestamp;

pub(crate) const GENESIS_BLOCK_HEIGHT: BlockHeight = 0;
pub(crate) const GENESIS_BLOCK_HASH: BlockHash = BlockHash::new([0; 32]);
pub(crate) const GENESIS_BLOCK_TIMESTAMP: Timestamp = 0;
