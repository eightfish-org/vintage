use crate::{BlockChainMsg, NetworkMsg, OverlordMsgBlock, ProxyMsg};
use tokio::sync::mpsc;

pub struct ProxyMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<ProxyMsg>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<BlockChainMsg>, // -> blockchain
}

pub struct BlockChainMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<BlockChainMsg>,
    // sender
    pub proxy_msg_sender: mpsc::Sender<ProxyMsg>, // -> proxy
    pub network_msg_sender: mpsc::Sender<NetworkMsg>, // -> network
}

pub struct ConsensusMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<OverlordMsgBlock>,
    // sender
    pub network_msg_sender: mpsc::Sender<NetworkMsg>, // -> blockchain
    pub consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
}

pub struct NetworkMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<NetworkMsg>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<BlockChainMsg>, // -> blockchain
}

pub fn msg_channels() -> (
    mpsc::Sender<ProxyMsg>,
    mpsc::Sender<BlockChainMsg>,
    mpsc::Sender<OverlordMsgBlock>,
    mpsc::Sender<NetworkMsg>,
    ProxyMsgChannels,
    BlockChainMsgChannels,
    ConsensusMsgChannels,
    NetworkMsgChannels,
) {
    // The maximum number of permits which a semaphore can hold.
    const BUFFER: usize = usize::MAX >> 3;

    let (proxy_msg_sender, proxy_msg_receiver) = mpsc::channel::<ProxyMsg>(BUFFER);
    let (blockchain_msg_sender, blockchain_msg_receiver) = mpsc::channel::<BlockChainMsg>(BUFFER);
    let (consensus_msg_sender, consensus_msg_receiver) = mpsc::channel::<OverlordMsgBlock>(BUFFER);
    let (network_msg_sender, network_msg_receiver) = mpsc::channel::<NetworkMsg>(BUFFER);

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
