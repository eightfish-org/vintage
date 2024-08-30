mod blockchain_test;

use self::blockchain_test::*;

use tokio::sync::mpsc;
use vintage_msg::{ MsgToBlockChain, MsgToNetwork, MsgToProxy, OverlordMsgBlock};

#[allow(dead_code)]
pub fn start_vintage_test(
    #[allow(unused_variables)] proxy_msg_sender: mpsc::Sender<MsgToProxy>,
    blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
    #[allow(unused_variables)] consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
    #[allow(unused_variables)] network_msg_sender: mpsc::Sender<MsgToNetwork>,
) {
    tokio::spawn(send_raw_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_act_to_blockchain(blockchain_msg_sender.clone()));
    tokio::spawn(send_block_to_blockchain(blockchain_msg_sender));
}
