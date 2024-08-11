use codec::BlockchainCodec;
use futures::SinkExt;
use futures::StreamExt;
use messages::{BlockchainMessage, NetworkMessage};
use peer_manager::{PeerInfo, PeerManager};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;
use vintage_msg::{BlockChainMsg, NetworkMsg, NetworkMsgChannels, OverlordMsgBlock};
pub mod codec;
pub mod config;
pub mod messages;
mod peer_manager;
use config::NodeConfig;

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub struct Node {
    address: SocketAddr,
    peers: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>>>,
    outgoing_messages: mpsc::Receiver<NetworkMsg>,
    incoming_messages: mpsc::Sender<BlockChainMsg>,
    consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
    peer_manager: Arc<PeerManager>,
}

impl Node {
    /*
    pub async fn new(
        config: NodeConfig,
    ) -> Result<
        (
            Self,
            mpsc::Sender<NetworkMessage>,
            mpsc::Receiver<NetworkMessage>,
        ),
        BoxedError,
    > {
        let (incoming_tx, incoming_rx) = mpsc::channel(100);
        let (outgoing_tx, outgoing_rx) = mpsc::channel(100);

        let peer_manager = Arc::new(PeerManager::new(5));
        for peer in &config.peers {
            peer_manager.add_peer(peer.address).await;
        }

        let node = Node {
            address: config.listen_addr,
            peers: Arc::new(Mutex::new(HashMap::new())),
            incoming_messages: incoming_tx,
            outgoing_messages: outgoing_rx,
            peer_manager,
        };

        Ok((node, outgoing_tx, incoming_rx))
    }
    */
    pub async fn create(
        config: &NodeConfig,
        channels: NetworkMsgChannels,
        consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
    ) -> Result<Self, anyhow::Error> {
        //let (incoming_tx, incoming_rx) = mpsc::channel(100);
        //let (outgoing_tx, outgoing_rx) = mpsc::channel(100);
        let outgoing_rx = channels.msg_receiver;
        let incoming_tx = channels.blockchain_msg_sender;
        let consensus_incoming_tx = consensus_msg_sender;
        let peer_manager = Arc::new(PeerManager::new(5));
        for peer in &config.peers {
            peer_manager.add_peer(peer.address).await;
        }

        let node = Node {
            address: config.listen_addr,
            peers: Arc::new(Mutex::new(HashMap::new())),
            incoming_messages: incoming_tx,
            outgoing_messages: outgoing_rx,
            consensus_incoming_messages: consensus_incoming_tx,
            peer_manager,
        };

        Ok(node)
    }
    pub fn start_service(self) -> JoinHandle<()> {
        // Node operation task
        let mut node = self;
        tokio::spawn(async move {
            node.start_peer_management().await;
            node.start().await;
        })
    }
    pub async fn start(&mut self) -> Result<(), BoxedError> {
        println!("Node will start and listening on: {}", self.address);
        let listener = TcpListener::bind(self.address).await?;
        println!("Node now listening on: {}", self.address);

        let incoming_messages = self.incoming_messages.clone();
        let consensus_incoming_messages = self.consensus_incoming_messages.clone();
        let peers = Arc::clone(&self.peers);
        let listen_addr = self.address;
        tokio::spawn(async move {
            while let Ok((socket, addr)) = listener.accept().await {
                println!("New connection from: {}", addr);
                if let Err(e) = Self::handle_connection(
                    socket,
                    addr,
                    listen_addr,
                    Arc::clone(&peers),
                    incoming_messages.clone(),
                    consensus_incoming_messages.clone(),
                )
                .await
                {
                    eprintln!("Error handling connection: {}", e);
                }
            }
        });

        self.message_loop().await
    }

    async fn message_loop(&mut self) -> Result<(), BoxedError> {
        loop {
            tokio::select! {
                Some(message) = self.outgoing_messages.recv() => {
                    //get a broad case message from application
                    let network_message = NetworkMessage{
                        sender: self.address,
                        content: BlockchainMessage::NetworkMsg(message)
                    };
                    self.handle_message(network_message).await?;
                }
            }
        }
    }

    async fn handle_message(&self, message: NetworkMessage) -> Result<(), BoxedError> {
        //println!("Processing message: {:.2?}", message);
        let formatted = format!("{:?}", message);
        log::info!("Processing outgoing message:: {:.100}", formatted);
        self.broadcast_message(message).await
    }

    async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), BoxedError> {
        let peers = self.peers.lock().await;
        for (peer_addr, tx) in peers.iter() {
            if *peer_addr != message.sender {
                // Don't send to the original sender
                if let Err(e) = tx.send(message.clone()).await {
                    eprintln!("Failed to send message to peer {}: {}", peer_addr, e);
                }
            }
        }
        // Send the message to the application layer
        //if let Err(e) = self.outgoing_messages.send(message).await {
        //    eprintln!("Failed to send message to application layer: {}", e);
        //}
        Ok(())
    }

    async fn handle_connection(
        socket: TcpStream,
        addr: SocketAddr,
        listening_addr: SocketAddr,
        peers: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>>>,
        incoming_messages: mpsc::Sender<BlockChainMsg>,
        consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
    ) -> Result<(), BoxedError> {
        let (tx, mut rx) = mpsc::channel::<NetworkMessage>(100);

        {
            //let mut peers = peers.lock().await;
            //peers.insert(addr, tx.clone());
        }

        let framed = Framed::new(socket, BlockchainCodec);
        let (mut sink, mut stream) = framed.split();

        // Send handshake
        let handshake = NetworkMessage {
            sender: listening_addr,
            content: BlockchainMessage::Handshake(listening_addr),
        };
        println!("Send out hand shake message to {}", addr);
        sink.send(handshake).await?;

        let mut peer_listening_addr = None;
        let incoming_messages = incoming_messages.clone();
        tokio::spawn(async move {
            log::info!("Processing incoming message from : {}", addr);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(message) => {
                        if let BlockchainMessage::Handshake(addr) = &message.content {
                            peer_listening_addr = Some(*addr);
                            let mut peers = peers.lock().await;
                            peers.insert(*addr, tx.clone());
                            log::info!("Handshake received from {}", addr);
                        } else if peer_listening_addr.is_some() {
                            let formatted = format!("{:?}", message);
                            log::info!(
                                "Received network message from {},data: {:.100}",
                                peer_listening_addr.unwrap(),
                                formatted
                            );
                            if let BlockchainMessage::NetworkMsg(msg) = &message.content {
                                if let NetworkMsg::BroadcastAct(act) = msg {
                                    let blockchain_msg = BlockChainMsg::ActFromNetwork(act.clone());
                                    // Use blockchain_msg here
                                    log::info!("Send BlockChainMsg::Act to vintage_blockchain");
                                    if let Err(e) = incoming_messages.send(blockchain_msg).await {
                                        eprintln!(
                                            "Failed to send message to application layer: {}",
                                            e
                                        );
                                        break;
                                    }
                                }
                                if let NetworkMsg::ConsensusMsg(consensus_msg) = msg {
                                    log::info!("Send ConsensusMsg to vintage_consensus");
                                    //let consensus_msg = OverlordMsg
                                    if let Err(e) = consensus_incoming_messages
                                        .send(consensus_msg.clone())
                                        .await
                                    {
                                        eprintln!(
                                            "Failed to send message to application layer: {}",
                                            e
                                        );
                                        break;
                                    }
                                }
                            } else {
                                println!("Message keep at network layer, skip the sending to application layer.")
                            }
                        } else {
                            println!("Received message before handshake from {}", addr);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket {}: {}", addr, e);
                        break;
                    }
                }
            }
            println!("Finished handling messages from {}", addr);
            if let Some(peer_addr) = peer_listening_addr {
                let mut peers = peers.lock().await;
                println!("remove addr: {} from peers", peer_addr);
                peers.remove(&peer_addr);
            }
        });

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = sink.send(message).await {
                    eprintln!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Ok(())
    }

    pub async fn connect_to_peer(&self, peer_addr: SocketAddr) -> Result<(), BoxedError> {
        if peer_addr == self.address {
            println!("Skipping self-connection to {}", peer_addr);
            return Ok(());
        }

        if self.peer_manager.is_connected(&peer_addr).await {
            println!("Already connected to {}", peer_addr);
            return Ok(());
        }

        let socket = TcpStream::connect(peer_addr).await?;
        println!("Connected to peer: {}", peer_addr);
        Self::handle_connection(
            socket,
            peer_addr,
            self.address,
            Arc::clone(&self.peers),
            self.incoming_messages.clone(),
            self.consensus_incoming_messages.clone(),
        )
        .await?;

        self.peer_manager.add_peer(peer_addr).await;
        Ok(())
    }

    pub async fn connect_to_peers(&self, peer_addrs: Vec<SocketAddr>) -> Result<(), BoxedError> {
        for addr in peer_addrs {
            if let Err(e) = self.connect_to_peer(addr).await {
                println!("Failed to connect to {}: {}", addr, e);
            }
        }
        Ok(())
    }

    pub async fn start_peer_management(&self) {
        let peer_manager = self.peer_manager.clone();
        let peers = self.peers.clone();
        let incoming_messages = self.incoming_messages.clone();
        let consensus_incoming_messages = self.consensus_incoming_messages.clone();
        let local_address = self.address.clone();
        tokio::spawn(async move {
            loop {
                // Health check
                for peer in peer_manager.get_peer_statuses().await {
                    if peer.connected_once {
                        let is_healthy = false; //check_peer_health(&peer.info.address).await;
                        peer_manager
                            .update_peer_status(peer.info.address, is_healthy)
                            .await;
                    }
                }

                // Reconnection attempts
                for peer in peer_manager.get_peers_to_reconnect().await {
                    println!("try to re-connect to: {}", peer.address);
                    if let Err(e) = reconnect_to_peer(
                        local_address,
                        &peer,
                        peers.clone(),
                        incoming_messages.clone(),
                        consensus_incoming_messages.clone(),
                    )
                    .await
                    {
                        eprintln!("Failed to reconnect to {}: {}", peer.address, e);
                    } else {
                        println!("re-connected to: {}", peer.address)
                    }
                }
                println!("reconnect completed.");
                // Remove unresponsive peers
                peer_manager.remove_unresponsive_peers().await;

                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await; // Run every 60 seconds
            }
        });
    }
}

async fn check_peer_health(addr: &SocketAddr) -> bool {
    tokio::net::TcpStream::connect(addr).await.is_ok()
}

async fn reconnect_to_peer(
    listening_addr: SocketAddr,
    peer: &PeerInfo,
    peers: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>>>,
    incoming_messages: mpsc::Sender<BlockChainMsg>,
    consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
) -> Result<(), BoxedError> {
    let socket = tokio::net::TcpStream::connect(peer.address).await?;
    println!("Reconnected to peer: {}", peer.address);
    Node::handle_connection(
        socket,
        peer.address,
        listening_addr,
        peers,
        incoming_messages,
        consensus_incoming_messages,
    )
    .await?;
    Ok(())
}
