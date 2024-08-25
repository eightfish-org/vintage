use crate::network::codec::{msg_encode, MsgKind};
use serde::Serialize;
use tokio::sync::mpsc;
use vintage_msg::MsgToNetwork;
use vintage_utils::SendMsg;

pub(crate) struct MsgToNetworkSender {
    sender: mpsc::Sender<MsgToNetwork>,
}

impl MsgToNetworkSender {
    pub fn new(sender: mpsc::Sender<MsgToNetwork>) -> Self {
        Self { sender }
    }

    pub fn broadcast<TMsg>(&self, msg: &TMsg) -> bool
    where
        TMsg: MsgKind + Serialize,
    {
        match msg_encode(msg) {
            Ok(msg_encoded) => self
                .sender
                .send_msg(MsgToNetwork::BlockChainMsg((None, msg_encoded))),
            Err(err) => {
                log::error!("failed to encode msg: {:?}", err);
                false
            }
        }
    }
}
