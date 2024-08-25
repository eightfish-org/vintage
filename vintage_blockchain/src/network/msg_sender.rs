use crate::network::msg_codec::{msg_encode, MsgKind};
use serde::Serialize;
use tokio::sync::mpsc;
use vintage_msg::{MsgToNetwork, NodeId};
use vintage_utils::SendMsg;

pub(crate) struct MsgToNetworkSender {
    sender: mpsc::Sender<MsgToNetwork>,
}

impl MsgToNetworkSender {
    pub fn new(sender: mpsc::Sender<MsgToNetwork>) -> Self {
        Self { sender }
    }

    pub fn broadcast_msg<TMsg>(&self, msg: &TMsg) -> bool
    where
        TMsg: MsgKind + Serialize,
    {
        self.send(None, msg)
    }

    pub fn send_msg<TMsg>(&self, node_id: NodeId, msg: &TMsg) -> bool
    where
        TMsg: MsgKind + Serialize,
    {
        self.send(Some(node_id), msg)
    }
}

impl MsgToNetworkSender {
    fn send<TMsg>(&self, node_id: Option<NodeId>, msg: &TMsg) -> bool
    where
        TMsg: MsgKind + Serialize,
    {
        match msg_encode(msg) {
            Ok(msg_encoded) => self
                .sender
                .send_msg(MsgToNetwork::BlockChainMsg((node_id, msg_encoded))),
            Err(err) => {
                log::error!("failed to encode msg: {:?}", err);
                false
            }
        }
    }
}
