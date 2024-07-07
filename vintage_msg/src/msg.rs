use crate::{Block, BlockProduction, Tx, TxId};

pub enum WorkerMsg {
    TxPersisted(Tx),
    TxDuplicated(TxId),
}

pub enum BlockChainMsg {
    RawTx(Tx), // tx from wasm worker
    Tx(Tx),    // tx from network
    Block(Block),
    BlockProduction(BlockProduction),
}

pub enum ConsensusMsg {}

pub enum NetworkMsg {
    BroadcastTx(Tx),
    BroadcastBlock(Block),
}