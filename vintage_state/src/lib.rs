mod db;

use crate::db::AsyncStateDb;
use vintage_msg::{EntityHash, EntityId, EntityName};

const STATE_DB_PATH: &str = "state.db";

pub struct StateMgr {
    db: AsyncStateDb,
}

impl StateMgr {
    pub async fn create() -> anyhow::Result<Self> {
        Ok(Self {
            db: AsyncStateDb::create(STATE_DB_PATH).await?,
        })
    }

    pub async fn get_entity_hash(
        &self,
        entity_name: EntityName,
        entity_id: EntityId,
    ) -> anyhow::Result<EntityHash> {
        self.db.get_entity_hash(entity_name, entity_id).await
    }

    pub async fn check_entity_hash(
        &self,
        entity_name: EntityName,
        pair_list: Vec<(EntityId, EntityHash)>,
    ) -> anyhow::Result<Option<EntityId>> {
        self.db.check_entity_hash(entity_name, pair_list).await
    }

    pub async fn write_entity_hash(
        &self,
        entity_name: EntityName,
        entity_id: EntityId,
        entity_hash: EntityHash,
    ) -> anyhow::Result<()> {
        self.db
            .write_entity_hash(entity_name, entity_id, entity_hash)
            .await
    }
}
