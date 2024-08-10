use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use vintage_msg::NetworkMsg;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BlockchainMessage {
    Handshake(SocketAddr),
    NewTransaction(Transaction),
    RawMessage(String), // Add this line
    NetworkMsg(NetworkMsg),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockRequest {
    pub start_index: u64,
    pub end_index: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMessage {
    pub sender: SocketAddr,
    pub content: BlockchainMessage,
}
