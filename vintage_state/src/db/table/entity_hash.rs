use anyhow::anyhow;
use redb::ReadableTable;
use sha2::{Digest, Sha256};
use vintage_msg::{EntityHash, EntityId, EntityName, Hashed};
use vintage_utils::{define_redb_table, RedbBytes32};

define_redb_table! {
    pub(crate) (EntityHashTable, EntityHashTableR, EntityHashTableW) = (RedbBytes32, RedbBytes32, "entity_hash")
}

impl<TABLE> EntityHashTable<TABLE>
where
    TABLE: ReadableTable<RedbBytes32, RedbBytes32>,
{
    pub fn get_entity_hash(
        &self,
        entity_name: &EntityName,
        entity_id: &EntityId,
    ) -> anyhow::Result<EntityHash> {
        let option = self.get(&entity_key_hashed(entity_name, &entity_id))?;
        match option {
            Some(access) => Ok(access.value().to_owned()),
            None => Err(anyhow!(
                "state {} not found",
                entity_key(entity_name, &entity_id)
            )),
        }
    }

    pub fn check_entity_hash(
        &self,
        entity_name: EntityName,
        pair_list: Vec<(EntityId, EntityHash)>,
    ) -> anyhow::Result<Option<EntityId>> {
        for (entity_id, entity_hash) in pair_list {
            let option = self.get(&entity_key_hashed(&entity_name, &entity_id))?;
            match option {
                Some(access) => {
                    if *access.value() != entity_hash {
                        return Ok(Some(entity_id.clone()));
                    }
                }
                None => return Ok(Some(entity_id.clone())),
            }
        }
        Ok(None)
    }
}

impl<'db, 'txn> EntityHashTableW<'db, 'txn> {
    pub fn insert_entity_hash(
        &mut self,
        entity_name: &EntityName,
        entity_id: &EntityId,
        entity_hash: &EntityHash,
    ) -> anyhow::Result<()> {
        self.insert(&entity_key_hashed(entity_name, entity_id), entity_hash)?;
        Ok(())
    }
}

fn entity_key(entity_name: &EntityName, entity_id: &EntityId) -> String {
    format!(
        "{}:{}:{}",
        entity_name.app_id, entity_name.table_name, entity_id
    )
}

fn entity_key_hashed(entity_name: &EntityName, entity_id: &EntityId) -> Hashed {
    let key = entity_key(entity_name, entity_id);
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.finalize().into()
}
