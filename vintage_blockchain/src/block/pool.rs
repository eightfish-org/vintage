use vintage_msg::{Block, BlockHeight, BlockProduction};
use vintage_utils::{Pool, WithId};

#[derive(Clone)]
pub(crate) enum BlockMsg {
    Block(Block),
    BlockProduction(BlockProduction),
}

impl WithId for BlockMsg {
    type Id = BlockHeight;

    fn id(&self) -> &Self::Id {
        match self {
            BlockMsg::Block(block) => &block.header.height,
            BlockMsg::BlockProduction(block_production) => &block_production.block_height,
        }
    }
}

pub(crate) type BlockMsgPool = Pool<BlockMsg>;
