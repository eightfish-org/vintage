use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerAddress {
    pub ip: IpAddr,
    pub port: Option<u16>,
}

impl From<SocketAddr> for PeerAddress {
    fn from(addr: SocketAddr) -> Self {
        PeerAddress {
            ip: addr.ip(),
            port: Some(addr.port()),
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub name: String,
    pub propose_weight: u32,
    pub vote_weight: u32,
}

#[derive(Debug, Clone)]
pub struct PeerStatus {
    pub info: PeerInfo,
    pub connected_once: bool,
    pub last_seen: Instant,
    pub failed_attempts: u32,
}

pub struct PeerManager {
    peers: Arc<Mutex<HashMap<PeerAddress, PeerStatus>>>,
    max_failed_attempts: u32,
}

impl PeerManager {
    pub fn new(max_failed_attempts: u32) -> Self {
        PeerManager {
            peers: Arc::new(Mutex::new(HashMap::new())),
            max_failed_attempts,
        }
    }

    pub async fn _load_peers_from_yaml(&self, yaml_content: &str) -> Result<(), serde_yaml::Error> {
        let peer_infos: Vec<PeerInfo> = serde_yaml::from_str(yaml_content)?;
        let mut peers = self.peers.lock().await;
        for info in peer_infos {
            println!("add peer: {}", info.address);
            peers.insert(
                info.address.into(),
                PeerStatus {
                    info: info.clone(),
                    last_seen: Instant::now(),
                    failed_attempts: 0,
                    connected_once: false,
                },
            );
        }
        Ok(())
    }

    pub async fn _get_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.lock().await;
        peers.values().map(|status| status.info.clone()).collect()
    }

    pub async fn get_peer_statuses(&self) -> Vec<PeerStatus> {
        let peers = self.peers.lock().await;
        peers.values().cloned().collect()
    }

    pub async fn update_peer_status(&self, addr: SocketAddr, is_healthy: bool) {
        println!("Update peer status: {}, {}", addr, is_healthy);
        let mut peers = self.peers.lock().await;
        if let Some(status) = peers.get_mut(&addr.into()) {
            if is_healthy {
                status.last_seen = Instant::now();
                status.failed_attempts = 0;
            } else {
                status.failed_attempts += 1;
            }
        }
    }

    pub async fn get_peers_to_reconnect(&self) -> Vec<PeerInfo> {
        let peers = self.peers.lock().await;
        for peer in peers.values() {
            println!(
                "Peer: {}, {}, {}",
                peer.info.address, peer.info.name, peer.failed_attempts
            );
        }
        peers
            .values()
            .filter(|status| {
                status.failed_attempts > 0 && status.failed_attempts <= self.max_failed_attempts
            })
            .map(|status| status.info.clone())
            .collect()
    }

    pub async fn remove_unresponsive_peers(&self) {
        let mut peers = self.peers.lock().await;
        peers.retain(|_, status| status.failed_attempts <= self.max_failed_attempts);
    }

    pub async fn add_peer(&self, addr: SocketAddr) -> bool {
        let mut peers = self.peers.lock().await;
        let peer_addr = PeerAddress::from(addr);
        if !peers.contains_key(&peer_addr) {
            peers.insert(
                peer_addr,
                PeerStatus {
                    connected_once: true,
                    info: PeerInfo {
                        address: addr,
                        name: format!("Node_{}", addr),
                        vote_weight: 0,
                        propose_weight: 0,
                    },
                    last_seen: Instant::now(),
                    failed_attempts: 0,
                },
            );
            true
        } else {
            false
        }
    }

    pub async fn is_connected(&self, addr: &SocketAddr) -> bool {
        let peers = self.peers.lock().await;
        let peer_addr = PeerAddress::from(*addr);
        peers.contains_key(&peer_addr)
    }
}
