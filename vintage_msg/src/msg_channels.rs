use crate::{MsgToBlockChain, MsgToNetwork, MsgToProxy, OverlordMsgBlock};
use tokio::sync::mpsc;

pub struct ProxyMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<MsgToProxy>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
}

pub struct BlockChainMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<MsgToBlockChain>,
    // sender
    pub proxy_msg_sender: mpsc::Sender<MsgToProxy>,
    pub network_msg_sender: mpsc::Sender<MsgToNetwork>,
}

pub struct ConsensusMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<OverlordMsgBlock>,
    // sender
    pub network_msg_sender: mpsc::Sender<MsgToNetwork>,
    pub consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
}

pub struct NetworkMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<MsgToNetwork>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
}

pub fn msg_channels() -> (
    mpsc::Sender<MsgToProxy>,
    mpsc::Sender<MsgToBlockChain>,
    mpsc::Sender<OverlordMsgBlock>,
    mpsc::Sender<MsgToNetwork>,
    ProxyMsgChannels,
    BlockChainMsgChannels,
    ConsensusMsgChannels,
    NetworkMsgChannels,
) {
    // The maximum number of permits which a semaphore can hold.
    const BUFFER: usize = usize::MAX >> 3;

    let (proxy_msg_sender, proxy_msg_receiver) = mpsc::channel::<MsgToProxy>(BUFFER);
    let (blockchain_msg_sender, blockchain_msg_receiver) = mpsc::channel::<MsgToBlockChain>(BUFFER);
    let (consensus_msg_sender, consensus_msg_receiver) = mpsc::channel::<OverlordMsgBlock>(BUFFER);
    let (network_msg_sender, network_msg_receiver) = mpsc::channel::<MsgToNetwork>(BUFFER);

    // channels
    (
        proxy_msg_sender.clone(),
        blockchain_msg_sender.clone(),
        consensus_msg_sender.clone(),
        network_msg_sender.clone(),
        ProxyMsgChannels {
            msg_receiver: proxy_msg_receiver,
            blockchain_msg_sender: blockchain_msg_sender.clone(),
        },
        BlockChainMsgChannels {
            msg_receiver: blockchain_msg_receiver,
            proxy_msg_sender: proxy_msg_sender,
            network_msg_sender: network_msg_sender.clone(),
        },
        ConsensusMsgChannels {
            msg_receiver: consensus_msg_receiver,
            network_msg_sender: network_msg_sender.clone(),
            consensus_msg_sender,
        },
        NetworkMsgChannels {
            msg_receiver: network_msg_receiver,
            blockchain_msg_sender,
        },
    )
}
