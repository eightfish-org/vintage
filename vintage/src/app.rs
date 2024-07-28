use tokio::task::JoinHandle;
use vintage_blockchain::BlockChain;
use vintage_consensus::Consensus;
use vintage_msg::{
    BlockChainMsgChannels, ConsensusMsgChannels, NetworkMsgChannels, StateMsgChannels,
    WorkerMsgChannels,
};
use vintage_network::{config::NodeConfig, Node};
use vintage_state::State;
use vintage_worker::Worker;

#[allow(dead_code)]
pub struct Vintage {
    worker: Worker,
    blockchain: BlockChain,
    consensus: Consensus,
    node: Node,
}

impl Vintage {
    pub async fn create(
        worker_chn: WorkerMsgChannels,
        state_chn: StateMsgChannels,
        blockchain_chn: BlockChainMsgChannels,
        consensus_chn: ConsensusMsgChannels,
        network_chn: NetworkMsgChannels,
        config: NodeConfig,
    ) -> anyhow::Result<Self> {
        let db_path = config.db_path.clone();

        let node = Node::create(config, network_chn).await?;
        let consensus = Consensus::create(consensus_chn).await?;
        #[allow(unused_variables)]
        let (blockchain, blockchain_api) = BlockChain::create(blockchain_chn, db_path).await?;
        #[allow(unused_variables)]
        let state_mgr = State::create(state_chn, blockchain_api).await?;
        let worker = Worker::create(worker_chn).await?;

        Ok(Self {
            worker,
            blockchain,
            consensus,
            node,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let blockchain = self.blockchain.start_service();
        let worker = self.worker.start_service();
        let node_service = self.node.start_service();
        tokio::spawn(async {
            let _ = blockchain.await;
            let _ = worker.await;
            let _ = node_service.await;
        })
    }
}
