use vintage_blockchain::BlockChain;
use vintage_consensus::Consensus;
use vintage_msg::msg_channels;
use vintage_network::Network;
use vintage_worker::Worker;

#[allow(dead_code)]
pub struct VintageApp {
    worker: Worker,
    blockchain: BlockChain,
    consensus: Consensus,
    network: Network,
}

impl VintageApp {
    #[allow(dead_code)]
    pub fn create() -> anyhow::Result<Self> {
        let (worker_chn, blockchain_chn, consensus_chn, network_chn) = msg_channels();
        let worker = Worker::create(worker_chn)?;
        let blockchain = BlockChain::create(blockchain_chn)?;
        let consensus = Consensus::create(consensus_chn)?;
        let network = Network::create(network_chn)?;
        Ok(Self {
            worker,
            blockchain,
            consensus,
            network,
        })
    }
}
