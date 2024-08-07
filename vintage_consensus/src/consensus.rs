#![allow(clippy::mutable_key_type)]

use vintage_msg::{
    BlockBody, BlockHeader, BlockProduction, ConsensusMsgChannels, NetworkMsg,
    SerializableOverlordMsg,
};
use vintage_network::config::NodeConfig;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use creep::Context;
use crossbeam_channel::unbounded;
use hasher::{Hasher, HasherKeccak};
use hummer::coding::hex_encode;
use lazy_static::lazy_static;
use rand::random;
use serde::{Deserialize, Serialize};

use overlord::error::ConsensusError;
use overlord::types::{Commit, Hash, Node, OverlordMsg, Status, ViewChangeReason};
use overlord::{Codec, Consensus, Crypto, DurationConfig, Overlord, OverlordHandler, Wal};
use tokio::sync::mpsc;
use vintage_msg::Block;
use std::net::SocketAddr;
lazy_static! {
    static ref HASHER_INST: HasherKeccak = HasherKeccak::new();
}

const SPEAKER_NUM: u8 = 10;

const SPEECH_INTERVAL: u64 = 1000; // ms

struct MockWal {
    inner: Mutex<Option<Bytes>>,
}

impl MockWal {
    fn new() -> MockWal {
        MockWal {
            inner: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Wal for MockWal {
    async fn save(&self, info: Bytes) -> Result<(), Box<dyn Error + Send>> {
        *self.inner.lock().unwrap() = Some(info);
        Ok(())
    }

    async fn load(&self) -> Result<Option<Bytes>, Box<dyn Error + Send>> {
        Ok(self.inner.lock().unwrap().as_ref().cloned())
    }
}

struct MockCrypto {
    name: Bytes,
}

impl MockCrypto {
    fn new(name: Bytes) -> Self {
        MockCrypto { name }
    }
}

impl Crypto for MockCrypto {
    fn hash(&self, speech: Bytes) -> Bytes {
        hash(&speech)
    }

    fn sign(&self, _hash: Bytes) -> Result<Bytes, Box<dyn Error + Send>> {
        Ok(self.name.clone())
    }

    fn aggregate_signatures(
        &self,
        _signatures: Vec<Bytes>,
        _speaker: Vec<Bytes>,
    ) -> Result<Bytes, Box<dyn Error + Send>> {
        Ok(Bytes::new())
    }

    fn verify_signature(
        &self,
        _signature: Bytes,
        _hash: Bytes,
        _voter: Bytes,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }

    fn verify_aggregated_signature(
        &self,
        _aggregated_signature: Bytes,
        _hash: Bytes,
        _voters: Vec<Bytes>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
}

struct ConsensusEngine {
    peer_list: Vec<Node>,
    outbound: mpsc::Sender<NetworkMsg>,
}

impl ConsensusEngine {
    fn new(peer_list: Vec<Node>, outbound: mpsc::Sender<NetworkMsg>) -> ConsensusEngine {
        ConsensusEngine {
            peer_list,
            outbound,
        }
    }
}

#[async_trait]
impl Consensus<Block> for ConsensusEngine {
    async fn get_block(
        &self,
        _ctx: Context,
        _height: u64,
    ) -> Result<(Block, Hash), Box<dyn Error + Send>> {
        let header = BlockHeader {
            height: 1,
            hash: [0; 32],

            timestamp: 1,
        };
        let body = BlockBody { acts: vec![] };
        let block = Block { header, body };
        Ok((block, "".into()))
        // TODO:
        // send BlockProduce
        // wait response and return
    }

    async fn check_block(
        &self,
        _ctx: Context,
        _height: u64,
        _hash: Hash,
        _speech: Block,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }

    async fn commit(
        &self,
        _ctx: Context,
        height: u64,
        commit: Commit<Block>,
    ) -> Result<Status, Box<dyn Error + Send>> {
        log::info!("=======block commit======");
        Ok(Status {
            height: height + 1,
            interval: Some(SPEECH_INTERVAL),
            timer_config: None,
            authority_list: self.peer_list.clone(),
        })
    }

    async fn get_authority_list(
        &self,
        _ctx: Context,
        _height: u64,
    ) -> Result<Vec<Node>, Box<dyn Error + Send>> {
        Ok(self.peer_list.clone())
    }

    async fn broadcast_to_other(
        &self,
        _ctx: Context,
        words: OverlordMsg<Block>,
    ) -> Result<(), Box<dyn Error + Send>> {
        self.outbound.send(NetworkMsg::ConsensusMsg(words)).await;
        Ok(())
    }

    async fn transmit_to_relayer(
        &self,
        _ctx: Context,
        name: Bytes,
        words: OverlordMsg<Block>,
    ) -> Result<(), Box<dyn Error + Send>> {
        //Skip for now
        Ok(())
    }

    fn report_error(&self, _ctx: Context, _err: ConsensusError) {}

    fn report_view_change(
        &self,
        _ctx: Context,
        _height: u64,
        _round: u64,
        _reason: ViewChangeReason,
    ) {
    }
}

pub struct Validator {
    overlord: Arc<Overlord<Block, ConsensusEngine, MockCrypto, MockWal>>,
    handler: OverlordHandler<Block>,
    consensus_engine: Arc<ConsensusEngine>,
    inbound: tokio::sync::Mutex<mpsc::Receiver<OverlordMsg<Block>>>,
}

impl Validator {
    pub fn new(
        config: &NodeConfig,
        outbound: mpsc::Sender<NetworkMsg>,
        inbound: mpsc::Receiver<OverlordMsg<Block>>, //this is our block chian or database.
    ) -> Self {
        log::info!("Validator Created.");
        let name = socket_addr_to_address(config.listen_addr);
        let node_list = build_node_list(config);
        let crypto = MockCrypto::new(name.clone());
        let consensus_engine = Arc::new(ConsensusEngine::new(node_list.clone(), outbound));
        let overlord = Overlord::new(
            name,
            Arc::clone(&consensus_engine),
            Arc::new(crypto),
            Arc::new(MockWal::new()),
        );
        let overlord_handler = overlord.get_handler();

        overlord_handler
            .send_msg(
                Context::new(),
                OverlordMsg::RichStatus(Status {
                    height: 1,
                    interval: Some(SPEECH_INTERVAL),
                    timer_config: None,
                    authority_list: node_list,
                }),
            )
            .unwrap();

        Self {
            overlord: Arc::new(overlord),
            handler: overlord_handler,
            consensus_engine,
            inbound: tokio::sync::Mutex::new(inbound),
        }
    }

    pub async fn run(
        self: Arc<Self>,
        config: NodeConfig
    ) -> Result<(), Box<dyn Error + Send>> {
        log::info!("==Validator run.");
        let interval = config.block_interval;
        let timer_config = timer_config();
        let node_list = build_node_list(&config);
        let brain = Arc::<ConsensusEngine>::clone(&self.consensus_engine);
        let handler = self.handler.clone();
        let s: Arc<Validator> = self.clone();
        let spawned_task = tokio::spawn(async move {
            log::info!("Validator Started.");
            loop {
                let msg = {
                    let mut receiver = s.inbound.lock().await;
                    receiver.recv().await
                };
                match msg {
                    Some(msg) => match msg {
                        OverlordMsg::SignedVote(vote) => {
                            handler
                                .send_msg(Context::new(), OverlordMsg::SignedVote(vote))
                                .unwrap();
                        }
                        OverlordMsg::SignedProposal(proposal) => {
                            handler
                                .send_msg(Context::new(), OverlordMsg::SignedProposal(proposal))
                                .unwrap();
                        }
                        OverlordMsg::AggregatedVote(agg_vote) => {
                            handler
                                .send_msg(Context::new(), OverlordMsg::AggregatedVote(agg_vote))
                                .unwrap();
                        }
                        OverlordMsg::SignedChoke(choke) => {
                            handler
                                .send_msg(Context::new(), OverlordMsg::SignedChoke(choke))
                                .unwrap();
                        }
                        _ => {}
                    },
                    None => {
                        eprintln!("receive nothing");
                        // Depending on the error type, you might want to break the loop here
                        // break;
                    }
                }
            }
        });

        self.clone()
            .overlord
            .run(0, interval,node_list,timer_config)
            .await
            .unwrap();
        spawned_task.await.unwrap();
        Ok(())
    }
}

fn gen_random_bytes() -> Bytes {
    let vec: Vec<u8> = (0..10).map(|_| random::<u8>()).collect();
    Bytes::from(vec)
}

fn hash(bytes: &Bytes) -> Bytes {
    let mut out = [0u8; 32];
    out.copy_from_slice(&HASHER_INST.digest(bytes));
    BytesMut::from(&out[..]).freeze()
}

pub fn timer_config() -> Option<DurationConfig> {
    Some(DurationConfig::new(10, 10, 10, 3))
}

fn socket_addr_to_address(addr: SocketAddr) -> Bytes {
    // Implementation depends on how you want to convert SocketAddr to Address
    // This is a placeholder implementation
    Bytes::from(addr.ip().to_string())
}

fn build_node_list(config: &NodeConfig) -> Vec<Node> {
    let mut nodes = Vec::new();

    // Add the current node
    nodes.push(Node {
        address: socket_addr_to_address(config.listen_addr),
        propose_weight: config.propose_weight,
        vote_weight: config.vote_weight,
    });

    // Add peer nodes
    for peer in &config.peers {
        nodes.push(Node {
            address: socket_addr_to_address(peer.address),
            propose_weight: peer.propose_weight,
            vote_weight: peer.vote_weight,
        });
    }

    nodes
}
