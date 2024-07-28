use crate::db::StateDb;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_msg::{ActId, BlockHeight, EntityHash, EntityKey, EntityState};

#[derive(Clone)]
pub(crate) struct AsyncStateDb {
    db: Arc<StateDb>,
}

// create
impl AsyncStateDb {
    pub async fn create(path: impl AsRef<Path> + Send + 'static) -> anyhow::Result<Self> {
        let db = spawn_blocking(|| StateDb::create(path)).await??;
        Ok(Self { db: Arc::new(db) })
    }
}

// read
impl AsyncStateDb {
    pub async fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_last_block_height()).await?
    }

    pub async fn get_acts_state(&self, act_ids: Vec<ActId>) -> anyhow::Result<Vec<EntityState>> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_acts_state(&act_ids)).await?
    }

    #[allow(dead_code)]
    pub async fn get_entity_state(&self, entity_key: EntityKey) -> anyhow::Result<EntityHash> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_entity_state(&entity_key)).await?
    }

    #[allow(dead_code)]
    pub async fn check_entity_state(
        &self,
        entity_key: EntityKey,
        entity_hash: EntityHash,
    ) -> anyhow::Result<bool> {
        let db = self.db.clone();
        spawn_blocking(move || db.check_entity_state(entity_key, entity_hash)).await?
    }
}

// write
impl AsyncStateDb {
    pub async fn insert_act_state(
        &self,
        act_id: ActId,
        entities_state: Vec<EntityState>,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.insert_act_state(act_id, &entities_state)).await?
    }

    pub async fn write_state(
        &self,
        block_height: BlockHeight,
        state_vec: Vec<EntityState>,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.write_state(block_height, state_vec)).await?
    }
}
