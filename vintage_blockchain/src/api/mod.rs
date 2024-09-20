use crate::BlockChainDb;
use async_trait::async_trait;
use vintage_msg::{BlockChainApi, BlockHeight, Entity, Model};

#[derive(Clone)]
pub struct BlockChainApiImpl {
    blockchain_db: BlockChainDb,
}

impl BlockChainApiImpl {
    pub(crate) fn new(blockchain_db: BlockChainDb) -> Self {
        Self { blockchain_db }
    }
}

#[async_trait]
impl BlockChainApi for BlockChainApiImpl {
    async fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        self.blockchain_db.get_block_height().await
    }

    async fn check_entities(&self, model: Model, entities: Vec<Entity>) -> bool {
        for entity in entities {
            match self
                .blockchain_db
                .get_entity(model.clone(), entity.id)
                .await
            {
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
