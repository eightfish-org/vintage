use crate::{Act, Block, BlockEvent, UpdateEntityTx};
use bytes::Bytes;
use overlord::types::OverlordMsg;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

pub enum MsgToBlockChain {
    // from network
    Request(NetworkRequestId, Vec<u8>),
    Broadcast(Vec<u8>),
    // from proxy
    Act(Act),
    UpdateEntityTx(UpdateEntityTx),
}

pub enum MsgToProxy {
    BlockEvent(BlockEvent),
}

pub type NodeId = SocketAddr;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMsgHandler {
    BlockChain,
    // Consensus,
}
pub type NetworkRequestId = u64;
pub type OverlordMsgBlock = OverlordMsg<Block>;
pub enum MsgToNetwork {
    Request(NodeId, NetworkMsgHandler, NetworkRequestId, Vec<u8>),
    RequestBroadcast(NetworkMsgHandler, NetworkRequestId, Vec<u8>),
    Broadcast(NetworkMsgHandler, Vec<u8>),
    ConsensusBroadcast(OverlordMsgBlock),
    ConsensusMsgRelay(Bytes, OverlordMsgBlock),
}
