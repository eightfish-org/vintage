use crate::{Act, ActEntitiesState, ActId, Block, BlockProduction};
use serde::{Deserialize, Serialize};

pub enum WorkerMsg {
    ActPersisted(Act),
    ActDuplicated(ActId),
}

pub enum StateMsg {
    // from wasm worker
    ActEntitiesState(ActEntitiesState),
}

pub enum BlockChainMsg {
    // from wasm worker
    RawAct(Act),
    // from network
    Act(Act),
    ImportBlock(Block),
    ProduceBlock(BlockProduction),
}

pub enum ConsensusMsg {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMsg {
    BroadcastAct(Act),
    BroadcastBlock(Block),
}
