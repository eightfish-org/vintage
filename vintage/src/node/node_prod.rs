use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_blockchain::BlockChain;
use vintage_consensus::{BlockConsensus, OverlordMsg, Validator};
use vintage_msg::{Block, NetworkMsg, NetworkMsgChannels, OverlordMsgBlock};
use vintage_network::config::NodeConfig;
use vintage_network::Node;
use vintage_utils::Service;

pub struct VintageNode {
    config: NodeConfig,
    node: Node,
    validator: Validator<BlockChain>,
}

impl VintageNode {
    pub async fn create(
        config: NodeConfig,
        network_chn: NetworkMsgChannels,
        consensus_msg_sender: mpsc::Sender<OverlordMsgBlock>,
        outbound: mpsc::Sender<NetworkMsg>,
        inbound: mpsc::Receiver<OverlordMsg<Block>>, //this is our blockchain or database.
        block_consensus: BlockChain,
    ) -> anyhow::Result<Self> {
        let block_height = block_consensus
            .get_block_height()
            .await
            .map_err(|err| anyhow!("get_block_height err: {:?}", err))?;

        let node = Node::create(&config, network_chn, consensus_msg_sender).await?;
        let validator = Validator::new(
            &config,
            outbound,
            inbound,
            block_consensus,
            block_height + 1,
        );

        Ok(Self {
            config,
            node,
            validator,
        })
    }
}

#[async_trait]
impl Service for VintageNode {
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
