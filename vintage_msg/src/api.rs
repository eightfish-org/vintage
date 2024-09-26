use crate::{BlockHeight, Entity, Model, Proto};
use async_trait::async_trait;

#[async_trait]
pub trait BlockChainApi {
    async fn get_block_height(&self) -> anyhow::Result<BlockHeight>;
    async fn check_entities(&self, proto: Proto, model: Model, entities: Vec<Entity>) -> bool;
}
