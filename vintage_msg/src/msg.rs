use crate::{ActTx, Block, BlockEvent, UpdateEntityTx, UploadWasm};
use bytes::Bytes;
use overlord::types::OverlordMsg;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////
// blockchain

pub enum MsgToBlockChain {
    // from network
    Broadcast(NodeId, Vec<u8>),
    Request(NodeId, NetworkRequestId, Vec<u8>),
    // from worker
    ActTx(ActTx),
    UpdateEntityTx(UpdateEntityTx),
    // from admin
    UploadWasm(UploadWasm),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// proxy

pub enum MsgToProxy {
    BlockEvent(BlockEvent),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// consensus

pub type OverlordMsgBlock = OverlordMsg<Block>;

////////////////////////////////////////////////////////////////////////////////////////////////////
// network

pub type NodeId = SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMsgHandler {
    BlockChain,
    // Consensus,
}

pub type NetworkRequestId = u64;

pub enum MsgToNetwork {
    Broadcast(NetworkMsgHandler, Vec<u8>),
    Request(NodeId, NetworkMsgHandler, NetworkRequestId, Vec<u8>),
    RequestBroadcast(NetworkMsgHandler, NetworkRequestId, Vec<u8>),
    Response(NodeId, NetworkRequestId, Vec<u8>),
    ConsensusBroadcast(OverlordMsgBlock),
    ConsensusMsgRelay(Bytes, OverlordMsgBlock),
}
