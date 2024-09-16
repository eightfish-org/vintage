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

impl NodeConfig {
    pub fn get_number_of_node(&self) -> usize {
        // Add peer nodes
        let number_of_peers = self.peers.len();
        number_of_peers + 1
    }
}
