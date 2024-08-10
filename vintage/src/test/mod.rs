mod blockchain_test;

use self::blockchain_test::*;

use tokio::sync::mpsc;
use vintage_consensus::OverlordMsg;
use vintage_msg::{Block, BlockChainMsg, NetworkMsg, ProxyMsg};

pub fn start_vintage_test(
    #[allow(unused_variables)] proxy_msg_sender: mpsc::Sender<ProxyMsg>,
    blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
    #[allow(unused_variables)] consensus_msg_sender: mpsc::Sender<OverlordMsg<Block>>,
    #[allow(unused_variables)] network_msg_sender: mpsc::Sender<NetworkMsg>,
) {
    tokio::spawn(send_raw_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_block_to_blockchain(blockchain_msg_sender));
}
