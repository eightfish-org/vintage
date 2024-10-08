pub mod client;
pub mod codec;
pub mod config;
pub mod messages;
mod peer_manager;
pub mod request;
mod response;

use crate::messages::{NetworkBroadcast, NetworkMessageContent, NetworkRequest, NetworkResponse};
use crate::request::ArcNetworkRequestMgr;
use bytes::Bytes;
use codec::BlockchainCodec;
use config::NodeConfig;
use futures::SinkExt;
use futures::StreamExt;
use messages::{NetworkMessage, NetworkMessagePayload};
use peer_manager::{PeerInfo, PeerManager};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;
use tokio_util::codec::Framed;
use vintage_msg::{
    MsgToBlockChain, MsgToNetwork, NetworkMsgChannels, NetworkMsgHandler, OverlordMsgBlock,
};

pub type BoxedError = Box<dyn std::error::Error + Send + Sync>;

pub struct Node {
    address: SocketAddr,
    peers: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>>>,
    outgoing_messages: mpsc::Receiver<MsgToNetwork>,
    incoming_messages: mpsc::Sender<MsgToBlockChain>,
    consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
    peer_manager: Arc<PeerManager>,
    request_mgr: ArcNetworkRequestMgr,
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
        request_mgr: ArcNetworkRequestMgr,
    ) -> Result<Self, anyhow::Error> {
        //let (incoming_tx, incoming_rx) = mpsc::channel(100);
        //let (outgoing_tx, outgoing_rx) = mpsc::channel(100);
        let outgoing_rx = channels.msg_receiver;
        let incoming_tx = channels.blockchain_msg_sender;
        let consensus_incoming_tx = channels.consensus_msg_sender;
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
            request_mgr,
        };

        Ok(node)
    }

    pub fn start_service(self) -> JoinHandle<()> {
        // Node operation task
        let mut node = self;
        tokio::spawn(async move {
            node.start_peer_management().await;
            let _ = node.start().await;
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
        let request_mgr = self.request_mgr.clone();
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
                    request_mgr.clone(),
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
            if let Some(message) = self.outgoing_messages.recv().await {
                match message {
                    MsgToNetwork::Broadcast(handler, broadcast_content) => {
                        let message_content = NetworkMessageContent::Broadcast(NetworkBroadcast {
                            handler,
                            broadcast_content,
                        });
                        self.handle_broadcast_message(message_content).await?;
                    }
                    MsgToNetwork::Request(node_id, handler, request_id, request_content) => {
                        let message_content = NetworkMessageContent::Request(NetworkRequest {
                            handler,
                            request_id,
                            request_content,
                        });
                        self.handle_message(node_id, message_content).await?;
                    }
                    MsgToNetwork::RequestBroadcast(handler, request_id, request_content) => {
                        let message_content = NetworkMessageContent::Request(NetworkRequest {
                            handler,
                            request_id,
                            request_content,
                        });
                        self.handle_broadcast_message(message_content).await?;
                    }
                    MsgToNetwork::Response(node_id, request_id, response_content) => {
                        let message_content = NetworkMessageContent::Response(NetworkResponse {
                            request_id,
                            response_content,
                        });
                        self.handle_message(node_id, message_content).await?;
                    }
                    MsgToNetwork::ConsensusBroadcast(content) => {
                        let message_content = NetworkMessageContent::ConsensusBroadcast(content);
                        self.handle_broadcast_message(message_content).await?;
                    }
                    MsgToNetwork::ConsensusMsgRelay(bytes, block) => {
                        let addr = bytes_to_socket_addr(&bytes);
                        match addr {
                            Ok(recovered_addr) => {
                                self.handle_message(
                                    recovered_addr,
                                    NetworkMessageContent::ConsensusMsgRelay(block),
                                )
                                .await?;
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                }
            }
        }
    }

    async fn handle_broadcast_message(
        &self,
        content: NetworkMessageContent,
    ) -> Result<(), BoxedError> {
        let message = NetworkMessage {
            sender: self.address,
            receiver: None,
            payload: NetworkMessagePayload::Content(content),
        };
        log::info!(
            "Processing outgoing message:: {:.100}",
            format!("{:?}", message)
        );
        self.broadcast_message(message).await
    }

    async fn handle_message(
        &self,
        receiver: SocketAddr,
        content: NetworkMessageContent,
    ) -> Result<(), BoxedError> {
        let message = NetworkMessage {
            sender: self.address,
            receiver: Some(receiver),
            payload: NetworkMessagePayload::Content(content),
        };
        log::info!(
            "Processing outgoing message:: {:.100}",
            format!("{:?}", message)
        );
        self.send_message(message).await
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

    async fn send_message(&self, message: NetworkMessage) -> Result<(), BoxedError> {
        let peers = self.peers.lock().await;
        match message.receiver {
            Some(receiver) => {
                // Send only to the specified receiver
                if let Some(tx) = peers.get(&receiver) {
                    if let Err(e) = tx.send(message.clone()).await {
                        eprintln!("Failed to send message to receiver {}: {}", receiver, e);
                    }
                } else {
                    eprintln!("Receiver {} not found in peers list", receiver);
                }
            }
            None => {}
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
        incoming_messages: mpsc::Sender<MsgToBlockChain>,
        consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
        request_mgr: ArcNetworkRequestMgr,
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
            receiver: Some(addr),
            payload: NetworkMessagePayload::Handshake(listening_addr),
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
                        if let NetworkMessagePayload::Handshake(addr) = message.payload {
                            peer_listening_addr = Some(addr);
                            let mut peers = peers.lock().await;
                            peers.insert(addr, tx.clone());
                            log::info!("Handshake received from {}", addr);
                        } else if peer_listening_addr.is_some() {
                            let formatted = format!("{:?}", message);
                            log::info!(
                                "Received network message from {},data: {:.100}",
                                peer_listening_addr.unwrap(),
                                formatted
                            );

                            if let NetworkMessagePayload::Content(content) = message.payload {
                                match content {
                                    NetworkMessageContent::Broadcast(broadcast) => {
                                        let msg = MsgToBlockChain::Broadcast(
                                            peer_listening_addr.unwrap(),
                                            broadcast.broadcast_content,
                                        );
                                        match broadcast.handler {
                                            NetworkMsgHandler::BlockChain => {
                                                log::info!("Send MsgToBlockChain::Broadcast to vintage_blockchain");
                                                if let Err(e) = incoming_messages.send(msg).await {
                                                    eprintln!(
                                                        "Failed to send message to application layer: {}",
                                                        e
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    NetworkMessageContent::Request(request) => {
                                        let msg = MsgToBlockChain::Request(
                                            peer_listening_addr.unwrap(),
                                            request.request_id,
                                            request.request_content,
                                        );
                                        match request.handler {
                                            NetworkMsgHandler::BlockChain => {
                                                log::info!("Send MsgToBlockChain::Request to vintage_blockchain");
                                                if let Err(e) = incoming_messages.send(msg).await {
                                                    eprintln!(
                                                        "Failed to send message to application layer: {}",
                                                        e
                                                    );
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    NetworkMessageContent::Response(response) => {
                                        request_mgr.lock().unwrap().on_response(
                                            peer_listening_addr.unwrap(),
                                            response.request_id,
                                            response.response_content,
                                        );
                                    }
                                    NetworkMessageContent::ConsensusBroadcast(consensus_msg) => {
                                        log::info!("Send ConsensusMsg to vintage_consensus");
                                        if let Err(err) =
                                            consensus_incoming_messages.send(consensus_msg).await
                                        {
                                            eprintln!(
                                                "Failed to send message to application layer: {}",
                                                err
                                            );
                                            break;
                                        }
                                    }
                                    NetworkMessageContent::ConsensusMsgRelay(consensus_msg) => {
                                        log::info!("Send ConsensusMsgRelay to vintage_consensus");
                                        if let Err(e) =
                                            consensus_incoming_messages.send(consensus_msg).await
                                        {
                                            eprintln!(
                                                "Failed to send message to application layer: {}",
                                                e
                                            );
                                            break;
                                        }
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
            self.request_mgr.clone(),
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
        let local_address = self.address.clone();
        let peer_manager = self.peer_manager.clone();
        let peers = self.peers.clone();
        let incoming_messages = self.incoming_messages.clone();
        let consensus_incoming_messages = self.consensus_incoming_messages.clone();
        let request_mgr = self.request_mgr.clone();
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
                        request_mgr.clone(),
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

                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                // Run every 60 seconds
            }
        });
    }
}

fn bytes_to_socket_addr(bytes: &Bytes) -> Result<SocketAddr, std::io::Error> {
    let addr_str = std::str::from_utf8(bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    addr_str
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

async fn _check_peer_health(addr: &SocketAddr) -> bool {
    tokio::net::TcpStream::connect(addr).await.is_ok()
}

async fn reconnect_to_peer(
    listening_addr: SocketAddr,
    peer: &PeerInfo,
    peers: Arc<Mutex<HashMap<SocketAddr, mpsc::Sender<NetworkMessage>>>>,
    incoming_messages: mpsc::Sender<MsgToBlockChain>,
    consensus_incoming_messages: mpsc::Sender<OverlordMsgBlock>,
    request_mgr: ArcNetworkRequestMgr,
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
        request_mgr,
    )
    .await?;
    Ok(())
}
