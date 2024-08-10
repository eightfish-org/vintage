use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{EntityHash, EntityId, Model};
use vintage_utils::{define_redb_table, RedbBytes32, RedbStr};

define_redb_table! {
    pub(crate) (EntityTable, EntityTableR, EntityTableW) = (RedbStr, RedbBytes32, "entity")
}

impl<TABLE> EntityTable<TABLE>
where
    TABLE: ReadableTable<RedbStr, RedbBytes32>,
{
    pub fn get_entity(&self, model: &Model, entity_id: &EntityId) -> anyhow::Result<EntityHash> {
        match self.get(format!("{}:{}", model, entity_id).as_str())? {
            Some(access) => Ok(access.value().into()),
            None => Err(anyhow!("entity {} {} not found", model, entity_id)),
        }
    }
}

impl<'db, 'txn> EntityTableW<'db, 'txn> {
    pub fn insert_entity(
        &mut self,
        model: &Model,
        entity_id: &EntityId,
        hash: &EntityHash,
    ) -> anyhow::Result<()> {
        self.insert(format!("{}:{}", model, entity_id).as_str(), hash.as_bytes())?;
        Ok(())
    }
}
