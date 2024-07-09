use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct Node {
    address: SocketAddr,
    peers: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
}

impl Node {
    pub async fn new(address: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            address,
            peers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(self.address).await?;
        println!("Node listening on: {}", self.address);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("New connection from: {}", addr);
            let peers = Arc::clone(&self.peers);
            tokio::spawn(async move {
                let mut peers = peers.lock().await;
                peers.insert(addr, socket);
            });
        }
    }

    pub async fn connect_to_peer(
        &self,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(peer_addr).await?;
        println!("Connected to peer: {}", peer_addr);
        let mut peers = self.peers.lock().await;
        peers.insert(peer_addr, stream);
        Ok(())
    }
}
