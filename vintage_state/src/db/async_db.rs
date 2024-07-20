use crate::db::StateDb;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use vintage_msg::{EntityHash, EntityId, EntityName};

#[derive(Clone)]
pub(crate) struct AsyncStateDb {
    pub db: Arc<Mutex<StateDb>>,
}

// create
impl AsyncStateDb {
    pub async fn create(path: impl AsRef<Path> + Send + 'static) -> anyhow::Result<Self> {
        let db = spawn_blocking(|| StateDb::create(path)).await??;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }
}

// read
impl AsyncStateDb {
    pub async fn get_entity_hash(
        &self,
        entity_name: EntityName,
        entity_id: EntityId,
    ) -> anyhow::Result<EntityHash> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().get_entity_hash(&entity_name, &entity_id)).await?
    }

    pub async fn check_entity_hash(
        &self,
        entity_name: EntityName,
        pair_list: Vec<(EntityId, EntityHash)>,
    ) -> anyhow::Result<Option<EntityId>> {
        let db = self.db.clone();
        spawn_blocking(move || db.lock().unwrap().check_entity_hash(entity_name, pair_list)).await?
    }
}

// write
impl AsyncStateDb {
    pub async fn write_entity_hash(
        &self,
        entity_name: EntityName,
        entity_id: EntityId,
        entity_hash: EntityHash,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || {
            db.lock()
                .unwrap()
                .write_entity_hash(&entity_name, &entity_id, &entity_hash)
        })
        .await?
    }
}
