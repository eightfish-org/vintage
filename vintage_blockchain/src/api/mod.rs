use crate::BlockChainDb;
use async_trait::async_trait;
use vintage_msg::{Block, BlockChainApi, BlockHeight, Entity, Proto};

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
    #[inline]
    async fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        self.blockchain_db.get_block_height().await
    }

    #[inline]
    async fn get_block(&self, block_height: BlockHeight) -> anyhow::Result<Block> {
        self.blockchain_db.get_block(block_height).await
    }

    async fn check_entities(&self, proto: Proto, entities: Vec<Entity>) -> bool {
        for entity in entities {
            match self
                .blockchain_db
                .get_entity(proto.clone(), entity.model, entity.id)
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
