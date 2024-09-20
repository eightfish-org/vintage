use crate::{ActTx, UpdateEntityTx, WasmTx};
use bytes::Bytes;
use overlord::Codec;
use serde::{Deserialize, Serialize};
use std::error::Error;
use vintage_utils::Hashed;

pub type BlockHeight = u64;
pub type BlockTimestamp = u64;
pub type BlockHash = Hashed;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub timestamp: BlockTimestamp,
    pub act_txs: Vec<ActTx>,
    pub ue_txs: Vec<UpdateEntityTx>,
    pub wasm_txs: Vec<WasmTx>,
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
