use crate::BlockChainDb;
use async_trait::async_trait;
use vintage_msg::{BlockChainApi, Entity, Model};

pub struct BlockChainApiImpl {
    db: BlockChainDb,
}

impl BlockChainApiImpl {
    pub(crate) fn new(db: BlockChainDb) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BlockChainApi for BlockChainApiImpl {
    async fn check_entities(&self, model: Model, entities: Vec<Entity>) -> bool {
        for entity in entities {
            match self.db.get_entity(model.clone(), entity.id).await {
                Ok(hash) => {
                    if hash != entity.hash {
                        return false;
                    }
                }
                Err(err) => {
                    log::error!("db get_entity err: {:?}", err);
                    return false;
                }
            }
        }
        true
    }
}
