use crate::{MsgToBlockChain, MsgToNetwork, MsgToProxy, OverlordMsgBlock};
use tokio::sync::mpsc;
use vintage_utils::MAX_MPSC_CHANNEL_SIZE;

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
    pub block_synced_sender: mpsc::Sender<u64>,
}

pub struct ConsensusMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<OverlordMsgBlock>,
    pub block_synced_receiver: mpsc::Receiver<u64>,
    // sender
    pub network_msg_sender: mpsc::Sender<MsgToNetwork>,
}

pub struct NetworkMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<MsgToNetwork>,
    // sender
    pub consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
    pub blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
}

pub fn msg_channels() -> (
    mpsc::Sender<MsgToBlockChain>,
    mpsc::Sender<MsgToProxy>,
    mpsc::Sender<OverlordMsgBlock>,
    mpsc::Sender<MsgToNetwork>,
    BlockChainMsgChannels,
    ProxyMsgChannels,
    ConsensusMsgChannels,
    NetworkMsgChannels,
) {
    let (proxy_msg_sender, proxy_msg_receiver) = mpsc::channel::<MsgToProxy>(MAX_MPSC_CHANNEL_SIZE);
    let (blockchain_msg_sender, blockchain_msg_receiver) =
        mpsc::channel::<MsgToBlockChain>(MAX_MPSC_CHANNEL_SIZE);
    let (consensus_msg_sender, consensus_msg_receiver) =
        mpsc::channel::<OverlordMsgBlock>(MAX_MPSC_CHANNEL_SIZE);
    let (network_msg_sender, network_msg_receiver) =
        mpsc::channel::<MsgToNetwork>(MAX_MPSC_CHANNEL_SIZE);
    let (block_synced_sender, block_synced_receiver) = mpsc::channel::<u64>(MAX_MPSC_CHANNEL_SIZE);
    // channels
    (
        blockchain_msg_sender.clone(),
        proxy_msg_sender.clone(),
        consensus_msg_sender.clone(),
        network_msg_sender.clone(),
        BlockChainMsgChannels {
            msg_receiver: blockchain_msg_receiver,
            proxy_msg_sender,
            network_msg_sender: network_msg_sender.clone(),
            block_synced_sender,
        },
        ProxyMsgChannels {
            msg_receiver: proxy_msg_receiver,
            blockchain_msg_sender: blockchain_msg_sender.clone(),
        },
        ConsensusMsgChannels {
            msg_receiver: consensus_msg_receiver,
            network_msg_sender,
            block_synced_receiver,
        },
        NetworkMsgChannels {
            msg_receiver: network_msg_receiver,
            blockchain_msg_sender,
            consensus_msg_sender,
        },
    )
}
