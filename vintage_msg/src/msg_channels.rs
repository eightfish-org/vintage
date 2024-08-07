use crate::{Block, BlockChainMsg, NetworkMsg, StateMsg, WorkerMsg};
use tokio::sync::mpsc;
use overlord::types::OverlordMsg;
pub struct WorkerMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<WorkerMsg>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<BlockChainMsg>, // -> blockchain
}

pub struct StateMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<StateMsg>,
}

pub struct BlockChainMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<BlockChainMsg>,
    // sender
    pub worker_msg_sender: mpsc::Sender<WorkerMsg>, // -> worker
    pub network_msg_sender: mpsc::Sender<NetworkMsg>, // -> network
}

pub struct ConsensusMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<OverlordMsg<Block>>,
    // sender
    pub network_msg_sender: mpsc::Sender<NetworkMsg>, // -> blockchain
}

pub struct NetworkMsgChannels {
    // receiver
    pub msg_receiver: mpsc::Receiver<NetworkMsg>,
    // sender
    pub blockchain_msg_sender: mpsc::Sender<BlockChainMsg>, // -> blockchain
}

pub fn msg_channels() -> (
    mpsc::Sender<WorkerMsg>,
    mpsc::Sender<BlockChainMsg>,
    mpsc::Sender<OverlordMsg<Block>>,
    mpsc::Sender<NetworkMsg>,
    WorkerMsgChannels,
    StateMsgChannels,
    BlockChainMsgChannels,
    ConsensusMsgChannels,
    NetworkMsgChannels,
) {
    // The maximum number of permits which a semaphore can hold.
    const BUFFER: usize = usize::MAX >> 3;

    let (worker_msg_sender, worker_msg_receiver) = mpsc::channel::<WorkerMsg>(BUFFER);
    #[allow(unused_variables)]
    let (state_msg_sender, state_msg_receiver) = mpsc::channel::<StateMsg>(BUFFER);
    let (blockchain_msg_sender, blockchain_msg_receiver) = mpsc::channel::<BlockChainMsg>(BUFFER);
    let (consensus_msg_sender, consensus_msg_receiver) = mpsc::channel::<OverlordMsg<Block>>(BUFFER);
    let (network_msg_sender, network_msg_receiver) = mpsc::channel::<NetworkMsg>(BUFFER);

    // channels
    (
        worker_msg_sender.clone(),
        blockchain_msg_sender.clone(),
        consensus_msg_sender.clone(),
        network_msg_sender.clone(),
        WorkerMsgChannels {
            msg_receiver: worker_msg_receiver,
            blockchain_msg_sender: blockchain_msg_sender.clone(),
        },
        StateMsgChannels {
            msg_receiver: state_msg_receiver,
        },
        BlockChainMsgChannels {
            msg_receiver: blockchain_msg_receiver,
            worker_msg_sender,
            network_msg_sender:network_msg_sender.clone(),
        },
        ConsensusMsgChannels {
            msg_receiver: consensus_msg_receiver,
            network_msg_sender: network_msg_sender.clone(),
        },
        NetworkMsgChannels {
            msg_receiver: network_msg_receiver,
            blockchain_msg_sender,
        },
    )
}
