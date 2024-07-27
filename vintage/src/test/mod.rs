mod blockchain_test;

use self::blockchain_test::*;

use tokio::sync::mpsc;
use vintage_msg::{BlockChainMsg, ConsensusMsg, NetworkMsg, WorkerMsg};

pub fn start_vintage_test(
    #[allow(unused_variables)] worker_msg_sender: mpsc::Sender<WorkerMsg>,
    blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
    #[allow(unused_variables)] consensus_msg_sender: mpsc::Sender<ConsensusMsg>,
    #[allow(unused_variables)] network_msg_sender: mpsc::Sender<NetworkMsg>,
) {
    tokio::spawn(send_raw_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_block_to_blockchain(blockchain_msg_sender));
}
