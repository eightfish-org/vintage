use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{EntityHash, EntityId, Model, Proto};
use vintage_utils::{define_redb_table, RedbStr};

define_redb_table! {
    pub(crate) (EntityTable, EntityTableR, EntityTableW) = (RedbStr, RedbStr, "entity")
}

impl<TABLE> EntityTable<TABLE>
where
    TABLE: ReadableTable<RedbStr, RedbStr>,
{
    pub fn get_entity(
        &self,
        proto: &Proto,
        model: &Model,
        entity_id: &EntityId,
    ) -> anyhow::Result<EntityHash> {
        match self.get(key(proto, model, entity_id).as_str())? {
            Some(access) => Ok(access.value().into()),
            None => Err(anyhow!("entity {} {} not found", model, entity_id)),
        }
    }
}

impl<'db, 'txn> EntityTableW<'db, 'txn> {
    pub fn insert_entity(
        &mut self,
        proto: &Proto,
        model: &Model,
        entity_id: &EntityId,
        hash: &EntityHash,
    ) -> anyhow::Result<()> {
        self.insert(key(proto, model, entity_id).as_str(), hash.as_str())?;
        Ok(())
    }
}

fn key(proto: &Proto, model: &Model, entity_id: &EntityId) -> String {
    format!("{}:{}:{}", proto, model, entity_id)
}
