use crate::db::{
    ActEntitiesStateTableR, ActEntitiesStateTableW, EntityStateTableR, EntityStateTableW,
    LastBlockHeightTableR, LastBlockHeightTableW, StateRootTableW,
};
use redb::Database;
use sha2::{Digest, Sha256};
use std::path::Path;
use vintage_msg::{
    ActId, BlockHeight, EntityHash, EntityKey, EntityState, StateRoot, GENESIS_BLOCK_HASH,
    GENESIS_BLOCK_HEIGHT,
};

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
        EntityStateTableW::open_table(&db_write)?;
        db_write.commit()?;
        Ok(())
    }
}

// read
impl StateDb {
    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db_read = self.database.begin_read()?;
        let table = LastBlockHeightTableR::open_table(&db_read)?;
        Ok(table.get_last_block_height()?)
    }

    pub fn get_acts_state(&self, ids: &[ActId]) -> anyhow::Result<Vec<EntityState>> {
        let db_read = self.database.begin_read()?;
        let table = ActEntitiesStateTableR::open_table(&db_read)?;
        table.get_acts_state(ids)
    }

    pub fn get_entity_state(&self, entity_key: &EntityKey) -> anyhow::Result<EntityHash> {
        let db_read = self.database.begin_read()?;
        let table = EntityStateTableR::open_table(&db_read)?;
        table.get_entity_state(entity_key)
    }

    pub fn check_entity_state(
        &self,
        entity_key: EntityKey,
        entity_hash: EntityHash,
    ) -> anyhow::Result<bool> {
        let db_read = self.database.begin_read()?;
        let table = EntityStateTableR::open_table(&db_read)?;
        table.check_entity_state(entity_key, entity_hash)
    }
}

// write
impl StateDb {
    pub fn insert_act_state(
        &self,
        act_id: ActId,
        entities_state: &Vec<EntityState>,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        let mut table = ActEntitiesStateTableW::open_table(&db_write)?;
        table.check_act_not_exists(act_id)?;
        table.insert_act_state(act_id, &entities_state)
    }

    pub fn write_state(
        &self,
        block_height: BlockHeight,
        state_vec: Vec<EntityState>,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        {
            // last state root
            let last_block_height = block_height - 1;
            let mut table_sr = StateRootTableW::open_table(&db_write)?;
            let last_state_root = if last_block_height == GENESIS_BLOCK_HEIGHT {
                GENESIS_BLOCK_HASH
            } else {
                table_sr.get_state_root(last_block_height)?
            };

            // calc new state root
            let mut hasher = Sha256::new();
            hasher.update(&last_state_root);

            let mut table_eh = EntityStateTableW::open_table(&db_write)?;
            for state in state_vec {
                table_eh.insert_entity_state(&state.entity_key, &state.entity_hash)?;

                // calc new state root
                hasher.update(&state.entity_key.to_string().as_bytes());
                hasher.update(&state.entity_hash);
            }

            let state_root: StateRoot = hasher.finalize().into();
            table_sr.insert(block_height, &state_root)?;
        }

        {
            let mut table_lbh = LastBlockHeightTableW::open_table(&db_write)?;
            table_lbh.insert((), block_height)?;
        }

        db_write.commit()?;
        Ok(())
    }
}
