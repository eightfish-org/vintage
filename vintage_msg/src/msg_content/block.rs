use crate::msg_content::Hashed;
use crate::{Act, UpdateEntityTx};
use bytes::Bytes;
use overlord::Codec;
use serde::{Deserialize, Serialize};
use std::error::Error;
pub type BlockHeight = u64;
pub type BlockTimestamp = u64;
pub type BlockHash = Hashed;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Block {
    pub timestamp: BlockTimestamp,
    pub acts: Vec<Act>,
    pub ue_txs: Vec<UpdateEntityTx>,
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
