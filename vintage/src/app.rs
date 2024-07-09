use tokio::task::JoinHandle;
use vintage_blockchain::BlockChain;
use vintage_consensus::Consensus;
use vintage_msg::{
    BlockChainMsgChannels, ConsensusMsgChannels, NetworkMsgChannels, WorkerMsgChannels,
};
use vintage_network::{config::NodeConfig, Node};
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
        blockchain_chn: BlockChainMsgChannels,
        consensus_chn: ConsensusMsgChannels,
        network_chn: NetworkMsgChannels,
        config: NodeConfig
    ) -> anyhow::Result<Self> {
        let worker = Worker::create(worker_chn).await?;
        let blockchain = BlockChain::create(blockchain_chn, config.db_path.clone()).await?;
        let consensus = Consensus::create(consensus_chn).await?;
        let node = Node::create(config, network_chn).await?;

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
