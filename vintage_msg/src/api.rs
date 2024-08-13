use crate::{BlockHeight, Entity, Model};
use async_trait::async_trait;

#[async_trait]
pub trait BlockChainApi {
    async fn get_block_height(&self) -> anyhow::Result<BlockHeight>;
    async fn check_entities(&self, model: Model, entities: Vec<Entity>) -> bool;
}
