use serde::{Deserialize, Serialize};
use vintage_msg::{Act, BlockHeight};

#[derive(Serialize, Deserialize)]
pub(crate) struct ReqBlockHash {
    pub begin_height: BlockHeight,
    pub count: usize,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ReqBlock {
    pub begin_height: BlockHeight,
    pub count: usize,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum RequestMsg {
    ReqBlockHash(ReqBlockHash),
    ReqBlock(ReqBlock),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum BroadcastMsg {
    Act(Act),
}
