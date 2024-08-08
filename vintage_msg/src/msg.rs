use crate::{Act, ActEntitiesState, ActId, Block, BlockProduction};
use bytes::Bytes;
use overlord::types::OverlordMsg;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializableOverlordMsg {
    SignedVote(Vec<u8>),
    SignedProposal(Vec<u8>),
    AggregatedVote(Vec<u8>),
    SignedChoke(Vec<u8>),
    RichStatus(Vec<u8>),
}
pub type OverlordMsgBlock = OverlordMsg<Block>;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NetworkMsg {
    BroadcastAct(Act),
    BroadcastBlock(Block),
    ConsensusMsg(OverlordMsgBlock),
}
