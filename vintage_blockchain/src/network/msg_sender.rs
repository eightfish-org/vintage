use crate::network::msgs::BroadcastMsg;
use serde::Serialize;
use tokio::sync::mpsc;
use vintage_msg::{MsgToNetwork, NetworkMsgHandler, NetworkRequestId, NodeId};
use vintage_utils::{BincodeSerialize, SendMsg};

#[derive(Clone)]
pub(crate) struct MsgToNetworkSender {
    sender: mpsc::Sender<MsgToNetwork>,
}

impl MsgToNetworkSender {
    pub fn new(sender: mpsc::Sender<MsgToNetwork>) -> Self {
        Self { sender }
    }

    pub fn send_broadcast(&self, msg: &BroadcastMsg) -> bool {
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

    pub fn send_response(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        msg: impl Serialize,
    ) -> bool {
        match msg.bincode_serialize() {
            Ok(msg_encoded) => {
                self.sender
                    .send_msg(MsgToNetwork::Response(node_id, request_id, msg_encoded))
            }
            Err(err) => {
                log::error!("failed to encode msg: {:?}", err);
                false
            }
        }
    }
}
