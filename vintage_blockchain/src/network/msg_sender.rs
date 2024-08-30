use crate::network::msgs::BroadcastMsg;
use tokio::sync::mpsc;
use vintage_msg::{MsgToNetwork, NetworkMsgHandler};
use vintage_utils::{BincodeSerialize, SendMsg};

#[derive(Clone)]
pub(crate) struct MsgToNetworkSender {
    sender: mpsc::Sender<MsgToNetwork>,
}

impl MsgToNetworkSender {
    pub fn new(sender: mpsc::Sender<MsgToNetwork>) -> Self {
        Self { sender }
    }

    pub fn broadcast_msg(&self, msg: &BroadcastMsg) -> bool {
        match msg.bincode_serialize() {
            Ok(msg_encoded) => self.sender.send_msg(MsgToNetwork::Broadcast(
                NetworkMsgHandler::BlockChain,
                msg_encoded,
            )),
            Err(err) => {
                log::error!("failed to encode msg: {:?}", err);
                false
            }
        }
    }
}
