use crate::network::{
    NetworkClientWrapper, ReqBlock, ReqBlockHash, RequestMsg, RspBlock, RspBlockHash,
};
use std::time::Duration;
use vintage_msg::{NetworkMsgHandler, NodeId};

pub(crate) struct BlockChainNetworkClient {
    client: NetworkClientWrapper,
}

impl BlockChainNetworkClient {
    pub fn new(client: NetworkClientWrapper) -> Self {
        Self { client }
    }

    const TIMEOUT: Duration = Duration::from_millis(10_000);

    pub async fn request_block_hash(
        &mut self,
        req: ReqBlockHash,
    ) -> anyhow::Result<(NodeId, RspBlockHash)> {
        self.client
            .request_broadcast_1(
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqBlockHash(req),
                Self::TIMEOUT,
            )
            .await
    }

    pub async fn request_block(
        &mut self,
        node_id: NodeId,
        req: ReqBlock,
    ) -> anyhow::Result<RspBlock> {
        self.client
            .request(
                node_id,
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqBlock(req),
                Self::TIMEOUT,
            )
            .await
    }
}
