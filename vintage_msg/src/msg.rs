use crate::{Act, Block, BlockEvent, UpdateEntityTx};
use overlord::types::OverlordMsg;
use serde::{Deserialize, Serialize};

pub enum BlockChainMsg {
    // from network
    ActFromNetwork(Act),

    // from proxy
    Act(Act),
    UpdateEntityTx(UpdateEntityTx),
}

pub enum ProxyMsg {
    BlockEvent(BlockEvent),
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
    ConsensusMsg(OverlordMsgBlock),
}
