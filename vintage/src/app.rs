use tokio::task::JoinHandle;
use vintage_blockchain::BlockChain;
use vintage_consensus::Consensus;
use vintage_msg::{
    BlockChainMsgChannels, ConsensusMsgChannels, NetworkMsgChannels, WorkerMsgChannels,
};
use vintage_network::Network;
use vintage_worker::Worker;

#[allow(dead_code)]
pub struct Vintage {
    worker: Worker,
    blockchain: BlockChain,
    consensus: Consensus,
    network: Network,
}

impl Vintage {
    pub async fn create(
        worker_chn: WorkerMsgChannels,
        blockchain_chn: BlockChainMsgChannels,
        consensus_chn: ConsensusMsgChannels,
        network_chn: NetworkMsgChannels,
    ) -> anyhow::Result<Self> {
        let worker = Worker::create(worker_chn).await?;
        let blockchain = BlockChain::create(blockchain_chn).await?;
        let consensus = Consensus::create(consensus_chn).await?;
        let network = Network::create(network_chn).await?;

        Ok(Self {
            worker,
            blockchain,
            consensus,
            network,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let blockchain = self.blockchain.start_service();
        let worker = self.worker.start_service();

        tokio::spawn(async {
            let _ = blockchain.await;
            let _ = worker.await;
        })
    }
}
