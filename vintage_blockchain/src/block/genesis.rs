use vintage_msg::{BlockHash, BlockHeight};
use vintage_utils::Bytes;

pub(crate) const GENESIS_BLOCK_HEIGHT: BlockHeight = 0;
pub(crate) const GENESIS_BLOCK_HASH: BlockHash = Bytes([0; 32]);
