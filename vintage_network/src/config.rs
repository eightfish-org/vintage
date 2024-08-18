use crate::peer_manager::PeerInfo;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConfig {
    pub listen_addr: SocketAddr,
    pub peers: Vec<PeerInfo>,
    pub name: String,
    pub propose_weight: u32,
    pub vote_weight: u32,
    pub block_interval: u64,
}
