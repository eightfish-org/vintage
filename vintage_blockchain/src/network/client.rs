use crate::network::{
    NetworkClientWrapper, ReqBlock, ReqBlockHash, RequestMsg, RspBlock, RspBlockHash,
};
use std::time::Duration;
use vintage_msg::{NetworkMsgHandler, NodeId, WasmHash};
use vintage_utils::BincodeSerialize;

pub struct BlockChainNetworkClient {
    client: NetworkClientWrapper,
}

impl BlockChainNetworkClient {
    pub(crate) fn new(client: NetworkClientWrapper) -> Self {
        Self { client }
    }

    const TIMEOUT: Duration = Duration::from_millis(10_000);

    pub(crate) async fn request_block_hash(
        &self,
        req: ReqBlockHash,
    ) -> anyhow::Result<(NodeId, RspBlockHash)> {
        self.client
            .request_with_vote_1(
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqBlockHash(req),
                Self::TIMEOUT,
            )
            .await
    }

    pub(crate) async fn request_block(
        &self,
        req: ReqBlock,
        node_id: NodeId,
    ) -> anyhow::Result<RspBlock> {
        self.client
            .request_with_single_node(
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqBlock(req),
                Self::TIMEOUT,
                node_id,
            )
            .await
    }

    pub(crate) async fn request_wasm_exists(
        &self,
        wasm_hash: WasmHash,
    ) -> anyhow::Result<(NodeId, bool)> {
        self.client
            .request_with_filter(
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqWasmExists(wasm_hash),
                Self::TIMEOUT,
                |data| is_true(data),
            )
            .await
    }

    pub(crate) async fn request_wasm(
        &self,
        wasm_hash: WasmHash,
        node_id: NodeId,
    ) -> anyhow::Result<Vec<u8>> {
        self.client
            .request_with_single_node(
                NetworkMsgHandler::BlockChain,
                RequestMsg::ReqWasm(wasm_hash),
                Duration::from_millis(120_000),
                node_id,
            )
            .await
    }
}

fn is_true(data: &[u8]) -> bool {
    data == true.bincode_serialize().unwrap()
}
