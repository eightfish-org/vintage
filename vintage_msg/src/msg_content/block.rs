use crate::msg_content::Hashed;
use crate::Act;
use serde::{Deserialize, Serialize};
use vintage_utils::WithId;
use bytes::{Bytes};
use overlord::{Codec};
use std::error::Error;
pub type BlockHeight = u64;
pub type BlockTimestamp = u64;
pub type BlockHash = Hashed;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct BlockBody {
    pub acts: Vec<Act>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProduction {
    pub block_height: BlockHeight,
}

macro_rules! impl_codec_for {
    ($($struc: ident),+) => {
        $(
            impl Codec for $struc {
                fn encode(&self) -> Result<Bytes, Box<dyn Error + Send>> {
                    Ok(Bytes::from(bincode::serialize(&self).unwrap()))
                    // serialize block to bytes
                }

                fn decode(data: Bytes) -> Result<Self, Box<dyn Error + Send>> {
                    // deserialize bytes to block
                    let data: Self = bincode::deserialize(&data).unwrap();
                    Ok(data)
                }
            }
        )+
    }
}

impl_codec_for!(Block);
