mod blockchain_dev;

use self::blockchain_dev::*;

use tokio::sync::mpsc;
use vintage_msg::MsgToBlockChain;

pub fn start_dev_task(node_name: &str, blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>) {
    // tokio::spawn(broadcast_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_act_to_blockchain(blockchain_msg_sender.clone()));
    if node_name == "Node1" {
        tokio::spawn(send_wasm_to_blockchain(blockchain_msg_sender.clone()));
    }
}
