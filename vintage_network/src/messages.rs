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
    Broadcast(NetworkBroadcast),
    Request(NetworkRequest),
    Response(NetworkResponse),
    ConsensusBroadcast(OverlordMsgBlock),
    ConsensusMsgRelay(OverlordMsgBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NetworkBroadcast {
    pub handler: NetworkMsgHandler,
    pub broadcast_content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NetworkRequest {
    pub handler: NetworkMsgHandler,
    pub request_id: NetworkRequestId,
    pub request_content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NetworkResponse {
    pub request_id: NetworkRequestId,
    pub response_content: Vec<u8>,
}
