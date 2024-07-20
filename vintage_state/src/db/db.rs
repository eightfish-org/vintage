use crate::db::entity_hash::{EntityHashTableR, EntityHashTableW};
use redb::Database;
use std::path::Path;
use vintage_msg::{EntityHash, EntityId, EntityName};

pub(crate) struct StateDb {
    database: Database,
}

// create
impl StateDb {
    pub fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = Self {
            database: Database::create(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        EntityHashTableW::open_table(&db_write)?;
        db_write.commit()?;
        Ok(())
    }
}

// read
impl StateDb {
    pub fn get_entity_hash(
        &self,
        entity_name: &EntityName,
        entity_id: &EntityId,
    ) -> anyhow::Result<EntityHash> {
        let db_read = self.database.begin_read()?;
        let table = EntityHashTableR::open_table(&db_read)?;
        table.get_entity_hash(entity_name, entity_id)
    }

    pub fn check_entity_hash(
        &self,
        entity_name: EntityName,
        pair_list: Vec<(EntityId, EntityHash)>,
    ) -> anyhow::Result<Option<EntityId>> {
        let db_read = self.database.begin_read()?;
        let table = EntityHashTableR::open_table(&db_read)?;
        table.check_entity_hash(entity_name, pair_list)
    }
}

// write
impl StateDb {
    pub fn write_entity_hash(
        &self,
        entity_name: &EntityName,
        entity_id: &EntityId,
        entity_hash: &EntityHash,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        {
            let mut table = EntityHashTableW::open_table(&db_write)?;
            table.insert_entity_hash(entity_name, entity_id, entity_hash)?;
        }
        db_write.commit()?;
        Ok(())
    }
}
