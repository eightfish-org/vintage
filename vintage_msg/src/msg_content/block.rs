use crate::Tx;
use serde::{Deserialize, Serialize};
use vintage_utils::WithId;

pub type BlockHeight = u64;
pub type BlockTimestamp = u64;
pub type BlockHash = [u8; 32]; // 256 bits

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockBody {
    pub txs: Vec<Tx>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl WithId for Block {
    type Id = BlockHeight;

    fn id(&self) -> &Self::Id {
        &self.header.height
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockProduction {
    pub block_height: BlockHeight,
}