use sha2::{Digest, Sha256};
use vintage_utils::{Bytes, Hashed};

pub type RowId = Hashed;
pub type RowHash = Hashed;

pub type AppId = String;
pub type TableName = String;
pub type EntityId = String;

pub fn row_id(app_id: AppId, table_name: TableName, entity_id: EntityId) -> Hashed {
    let row_id = format!("{}:{}:{}", app_id, table_name, entity_id);
    let mut hasher = Sha256::new();
    hasher.update(row_id.as_bytes());
    Bytes(hasher.finalize().into())
}
