use crate::db::MetaDataDb;
use async_trait::async_trait;
use std::time::Duration;
use vintage_msg::{BlockChainApi, BlockHeight, Entity};
use vintage_utils::{CalcHash, Service, ServiceStarter};

pub struct MetaDataService<TApi> {
    db: MetaDataDb,
    blockchain_api: TApi,
}

impl<TApi> MetaDataService<TApi>
where
    TApi: BlockChainApi + Send + Sync + 'static,
{
    pub fn create(db: MetaDataDb, blockchain_api: TApi) -> ServiceStarter<Self> {
        ServiceStarter::new(Self { db, blockchain_api })
    }

    async fn sync_block(&self, block_height_ref: &mut BlockHeight) -> anyhow::Result<bool> {
        let block_height = *block_height_ref + 1;

        // block
        let block = match self.blockchain_api.get_block(block_height).await {
            Ok(block) => block,
            Err(err) => {
                log::debug!("blockchain_api.get_block: {:?}", err);
                return Ok(false);
            }
        };

        // entities
        let mut entities = Vec::new();
        for ue_tx in block.ue_txs {
            let tx_id = ue_tx.calc_hash();
            for (index, entity) in ue_tx.entities.into_iter().enumerate() {
                entities.push((
                    tx_id.clone(),
                    index,
                    ue_tx.proto.clone(),
                    Entity {
                        model: entity.model,
                        id: entity.id,
                        hash: entity.hash,
                    },
                ))
            }
        }

        // sql_migrations
        let mut sql_migrations = Vec::new();
        for wasm_tx in block.wasm_txs {
            sql_migrations.push((
                wasm_tx.calc_hash(),
                wasm_tx.proto,
                wasm_tx.sql,
                wasm_tx.after_blocks,
            ))
        }

        self.db
            .save_block(block_height, block.timestamp, entities, sql_migrations)
            .await?;
        *block_height_ref = block_height;
        Ok(true)
    }
}

#[async_trait]
impl<TApi> Service for MetaDataService<TApi>
where
    TApi: BlockChainApi + Send + Sync + 'static,
{
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        let mut block_height = match self.db.query_block_height().await {
            Err(_) => return,
            Ok(block_height) => block_height,
        };
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            loop {
                match self.sync_block(&mut block_height).await {
                    Ok(success) => {
                        if !success {
                            break;
                        }
                    }
                    Err(err) => {
                        log::error!("MetaDataService error: {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}
