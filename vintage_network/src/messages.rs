use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use vintage_msg::{NetworkMsgHandler, NetworkRequestId, OverlordMsgBlock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NetworkMessage {
    pub sender: SocketAddr,
    pub receiver: Option<SocketAddr>,
    pub payload: NetworkMessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum NetworkMessagePayload {
    Handshake(SocketAddr),
    Content(NetworkMessageContent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum NetworkMessageContent {
    Request(NetworkRequest),
    Response(NetworkResponse),
    Broadcast(NetworkBroadcast),
    ConsensusBroadcast(OverlordMsgBlock),
    ConsensusMsgRelay(Bytes, OverlordMsgBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub handler: NetworkMsgHandler,
    pub request_id: NetworkRequestId,
    pub request_content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub request_id: NetworkRequestId,
    pub response_content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBroadcast {
    pub handler: NetworkMsgHandler,
    pub broadcast_content: Vec<u8>,
}
