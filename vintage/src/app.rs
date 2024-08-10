use std::sync::Arc;
use tokio::task::JoinHandle;
use vintage_blockchain::{BlockChain, BlockChainApiImpl, TxService};
use vintage_consensus::Validator;
use vintage_msg::{
    BlockChainMsgChannels, ConsensusMsgChannels, NetworkMsgChannels, ProxyMsgChannels,
};
use vintage_network::{config::NodeConfig, Node};
use vintage_proxy::Proxy;
use vintage_utils::start_service;

#[allow(dead_code)]
pub struct Vintage {
    tx_service: TxService,
    proxy: Proxy<BlockChainApiImpl>,
    validator: Validator<BlockChain>,
    node: Node,
    config: NodeConfig,
}

impl Vintage {
    pub async fn create(
        proxy_chn: ProxyMsgChannels,
        blockchain_chn: BlockChainMsgChannels,
        consensus_chn: ConsensusMsgChannels,
        network_chn: NetworkMsgChannels,
        config: NodeConfig,
    ) -> anyhow::Result<Self> {
        let (blockchain, blockchain_api, tx_service) =
            BlockChain::create(blockchain_chn, config.db_path.clone()).await?;
        let proxy = Proxy::create(proxy_chn, blockchain_api).await?;

        let node = Node::create(&config, network_chn, consensus_chn.consensus_msg_sender).await?;
        let validator = Validator::new(
            &config,
            consensus_chn.network_msg_sender,
            consensus_chn.msg_receiver,
            blockchain,
        );

        Ok(Self {
            tx_service,
            proxy,
            validator,
            node,
            config,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let tx_service = start_service(self.tx_service, ());
        let proxy_inbound = start_service(self.proxy.inbound, self.proxy.pub_sub);
        let proxy_outbound = start_service(self.proxy.outbound, ());

        let validator = Arc::new(self.validator);
        let validator_service = validator.run(self.config.clone());
        let node_service = self.node.start_service();

        tokio::spawn(async {
            let _ = tx_service.await;
            let _ = proxy_inbound.await;
            let _ = proxy_outbound.await;
            let _ = validator_service.await;
            let _ = node_service.await;
        })
    }
}
