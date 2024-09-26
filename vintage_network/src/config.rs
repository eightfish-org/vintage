use crate::peer_manager::PeerInfo;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConfig {
    pub block_interval: u64,
    pub id: u16,
    pub name: String,
    pub listen_addr: SocketAddr,
    pub peers: Vec<PeerInfo>,
    pub propose_weight: u32,
    pub vote_weight: u32,
}

impl NodeConfig {
    pub fn get_number_of_node(&self) -> usize {
        // Add peer nodes
        let number_of_peers = self.peers.len();
        number_of_peers + 1
    }
}
