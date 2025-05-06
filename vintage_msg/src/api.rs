use crate::{Block, BlockHeight, Entity, Proto};
use async_trait::async_trait;

#[async_trait]
pub trait BlockChainApi {
    async fn get_block_height(&self) -> anyhow::Result<BlockHeight>;
    async fn get_block(&self, block_height: BlockHeight) -> anyhow::Result<Block>;
    async fn check_entities(&self, proto: Proto, entities: Vec<Entity>) -> bool;
}
