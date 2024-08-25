use crate::{Act, Block, BlockEvent, UpdateEntityTx};
use bytes::Bytes;
use overlord::types::OverlordMsg;
use serde::{Deserialize, Serialize};

pub type NodeId = String;

pub enum MsgToBlockChain {
    // from network
    NetworkMsg((NodeId, Vec<u8>)),

    // from proxy
    Act(Act),
    UpdateEntityTx(UpdateEntityTx),
}

pub enum MsgToProxy {
    BlockEvent(BlockEvent),
}

pub type OverlordMsgBlock = OverlordMsg<Block>;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MsgToNetwork {
    BlockChainMsg((Option<NodeId>, Vec<u8>)),
    ConsensusMsg(OverlordMsgBlock),
    ConsensusMsgRelay((Bytes, OverlordMsgBlock)),
}
