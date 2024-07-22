use crate::{Block, BlockProduction, Tx, TxId};
use serde::{Deserialize, Serialize};
pub enum WorkerMsg {
    TxPersisted(Tx),
    TxDuplicated(TxId),
}

pub enum BlockChainMsg {
    RawTx(Tx), // tx from wasm worker
    Tx(Tx),    // tx from network
    ImportBlock(Block),
    ProduceBlock(BlockProduction),
}

pub enum ConsensusMsg {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMsg {
    BroadcastTx(Tx),
    BroadcastBlock(Block),
}
