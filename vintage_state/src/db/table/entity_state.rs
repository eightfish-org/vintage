use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{EntityHash, EntityKey};
use vintage_utils::{define_redb_table, RedbBytesN, RedbStr};

define_redb_table! {
    pub(crate) (EntityStateTable, EntityStateTableR, EntityStateTableW) = (RedbStr, RedbBytesN<32>, "entity_state")
}

impl<TABLE> EntityStateTable<TABLE>
where
    TABLE: ReadableTable<RedbStr, RedbBytesN<32>>,
{
    pub fn get_entity_state(&self, entity_key: &EntityKey) -> anyhow::Result<EntityHash> {
        match self.get(&*entity_key.to_string())? {
            Some(access) => Ok(access.value().to_owned()),
            None => Err(anyhow!("entity {} not found", entity_key)),
        }
    }

    pub fn check_entity_state(
        &self,
        entity_key: EntityKey,
        entity_hash: EntityHash,
    ) -> anyhow::Result<bool> {
        Ok(match self.get(&*entity_key.to_string())? {
            Some(access) => *access.value() == entity_hash,
            None => false,
        })
    }
}

impl<'db, 'txn> EntityStateTableW<'db, 'txn> {
    pub fn insert_entity_state(
        &mut self,
        entity_key: &EntityKey,
        entity_hash: &EntityHash,
    ) -> anyhow::Result<()> {
        self.insert(&*entity_key.to_string(), entity_hash)?;
        Ok(())
    }
}
