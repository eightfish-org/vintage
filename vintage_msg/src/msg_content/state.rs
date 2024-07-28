use crate::msg_content::Hashed;
use crate::ActId;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// entity key
#[derive(Serialize, Deserialize)]
pub struct EntityKey {
    pub app_id: AppId,
    pub table_name: TableName,
    pub entity_id: EntityId,
}
pub type AppId = String;
pub type TableName = String;
pub type EntityId = String;

impl Display for EntityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.app_id, self.table_name, self.entity_id)
    }
}

// entity hash
pub type EntityHash = Hashed;

// entity key & hash
#[derive(Serialize, Deserialize)]
pub struct EntityState {
    pub entity_key: EntityKey,
    pub entity_hash: EntityHash,
}

// act entities key & hash
pub struct ActEntitiesState {
    pub act_id: ActId,
    pub entities_state: Vec<EntityState>,
}

// state root
pub type StateRoot = Hashed;
