use crate::network::{ReqBlock, ReqBlockHash, RequestMsg, RspBlock, RspBlockHash};
use std::time::Duration;
use vintage_msg::NetworkMsgHandler;
use vintage_network::{client::NetworkClient, config::NodeConfig};
use vintage_utils::{BincodeDeserialize, BincodeSerialize};

pub(crate) struct BlockChainNetworkClient {
    client: NetworkClient,
    node_config: NodeConfig
}

impl BlockChainNetworkClient {
    pub fn new(client: NetworkClient, node_config: NodeConfig) -> Self {
        Self { client, node_config }
    }

    const TIMEOUT: Duration = Duration::from_millis(10_000);

    pub async fn request_block_hash(&mut self, req: ReqBlockHash) -> anyhow::Result<RspBlockHash> {
        let rsp_encoded = self
            .request_broadcast(Self::TIMEOUT, RequestMsg::ReqBlockHash(req))
            .await?;
        let (rsp, _bytes_read) = RspBlockHash::bincode_deserialize(&rsp_encoded)?;
        Ok(rsp)
    }

    pub async fn request_block(&mut self, req: ReqBlock) -> anyhow::Result<RspBlock> {
        let rsp_encoded = self
            .request_broadcast(Self::TIMEOUT, RequestMsg::ReqBlock(req))
            .await?;
        let (rsp, _bytes_read) = RspBlock::bincode_deserialize(&rsp_encoded)?;
        Ok(rsp)
    }

    async fn request_broadcast(
        &mut self,
        timeout: Duration,
        request: RequestMsg,
    ) -> anyhow::Result<Vec<u8>> {
        let encoded = request.bincode_serialize()?;
        let rsp_encoded = self
            .client
            .request_broadcast(timeout, NetworkMsgHandler::BlockChain, encoded, self.node_config.get_number_of_node()) // todo node_count
            .await?;
        Ok(rsp_encoded)
    }
}
