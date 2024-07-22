use crate::peer_manager::PeerInfo;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
#[derive(Debug, Deserialize, Serialize)]
pub struct NodeConfig {
    pub listen_addr: SocketAddr,
    pub peers: Vec<PeerInfo>,
    pub db_path: String,
}
