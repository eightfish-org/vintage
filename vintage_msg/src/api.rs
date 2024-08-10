use crate::{Entity, Model};
use async_trait::async_trait;

#[async_trait]
pub trait BlockChainApi {
    async fn check_entities(&self, model: Model, entities: Vec<Entity>) -> bool;
}
