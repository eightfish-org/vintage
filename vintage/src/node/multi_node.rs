use async_trait::async_trait;
use std::sync::Arc;
use vintage_blockchain::BlockConsensusImpl;
use vintage_consensus::Validator;
use vintage_msg::{ConsensusMsgChannels, NetworkMsgChannels};
use vintage_network::config::NodeConfig;
use vintage_network::request::ArcNetworkRequestMgr;
use vintage_network::Node;
use vintage_utils::{Service, ServiceStarter};

pub struct VintageMultiNode {
    config: NodeConfig,
    validator: Validator<BlockConsensusImpl>,
    node: Node,
}

impl VintageMultiNode {
    pub async fn create(
        config: NodeConfig,
        consensus_chn: ConsensusMsgChannels,
        network_chn: NetworkMsgChannels,
        block_consensus: BlockConsensusImpl,
        request_mgr: ArcNetworkRequestMgr,
    ) -> anyhow::Result<ServiceStarter<Self>> {
        let node = Node::create(&config, network_chn, request_mgr).await?;

        let validator = Validator::create(&config, consensus_chn, block_consensus).await?;

        Ok(ServiceStarter::new(Self {
            config,
            node,
            validator,
        }))
    }
}

#[async_trait]
impl Service for VintageMultiNode {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        let validator_service = Arc::new(self.validator).run(self.config.clone());
        let node_service = self.node.start_service();

        if let Err(err) = validator_service.await {
            log::error!("Validator service error: {:?}", err)
        }
        if let Err(err) = node_service.await {
            log::error!("Node service error: {:?}", err)
        }
    }
}
