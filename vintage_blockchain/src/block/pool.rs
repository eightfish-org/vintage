use vintage_msg::{Block, BlockHeight, BlockProduction};
use vintage_utils::{Pool, WithId};

#[derive(Clone)]
pub(crate) enum BlockMsg {
    ImportBlock(Block),
    ProduceBlock(BlockProduction),
}

impl WithId for BlockMsg {
    type Id = BlockHeight;

    fn id(&self) -> &Self::Id {
        match self {
            BlockMsg::ImportBlock(block) => &block.header.height,
            BlockMsg::ProduceBlock(block_production) => &block_production.block_height,
        }
    }
}

pub(crate) type BlockMsgPool = Pool<BlockMsg>;
