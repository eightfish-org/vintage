mod blockchain_test;

use self::blockchain_test::*;

use tokio::sync::mpsc;
use vintage_msg::MsgToBlockChain;

pub fn start_test(_node_name: &str, blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>) {
    // tokio::spawn(broadcast_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_act_tx_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_ue_tx_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_wasm_tx_to_blockchain(blockchain_msg_sender));
}
