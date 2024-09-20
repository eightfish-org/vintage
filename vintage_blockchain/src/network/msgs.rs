use serde::{Deserialize, Serialize};
use vintage_msg::{ActTx, Block, BlockHash, BlockHeight};

#[derive(Serialize, Deserialize)]
pub(crate) enum BroadcastMsg {
    ActTx(ActTx),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum RequestMsg {
    ReqBlockHash(ReqBlockHash),
    ReqBlock(ReqBlock),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReqBlockHash {
    pub begin_height: BlockHeight,
    pub count: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReqBlock {
    pub begin_height: BlockHeight,
    pub count: u64,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RspBlockHash {
    pub hash_list: Vec<BlockHash>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RspBlock {
    pub block_list: Vec<Block>,
}
