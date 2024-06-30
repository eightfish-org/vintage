use crate::{Block, BlockProduction, Tx, TxId};

pub enum WorkerMsg {
    TxPersisted(Tx),
    TxDuplicated(TxId),
}

pub enum BlockChainMsg {
    TxFromWorker(Tx),
    TxFromNetwork(Tx),
    Block(Block),
    BlockProduction(BlockProduction),
}

pub enum ConsensusMsg {}

pub enum NetworkMsg {
    BroadcastTx(Tx),
    BroadcastBlock(Block),
}
