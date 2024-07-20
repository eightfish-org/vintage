use crate::msg_content::Hashed;

// key
pub type AppId = String;
pub type TableName = String;
pub type EntityId = String;

pub struct EntityName {
    pub app_id: AppId,
    pub table_name: TableName,
}

// value
pub type EntityHash = Hashed;
