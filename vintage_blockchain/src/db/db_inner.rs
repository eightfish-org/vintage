use crate::chain::BlockState;
use crate::db::{
    ActTableR, ActTableW, BlockInDb, BlockTableR, BlockTableW, LastBlockHeightTableR,
    LastBlockHeightTableW,
};
use crate::tx::ActId;
use redb::Database;
use std::path::Path;
use vintage_msg::{Block, BlockHash, BlockHeight};

pub(crate) struct BlockChainDbInner {
    database: Database,
}

// create
impl BlockChainDbInner {
    pub fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = Self {
            database: Database::create(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        ActTableW::open_table(&db_write)?;
        LastBlockHeightTableW::open_table(&db_write)?;
        BlockTableW::open_table(&db_write)?;
        db_write.commit()?;
        Ok(())
    }
}

// read
impl BlockChainDbInner {
    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db_read = self.database.begin_read()?;
        let table = LastBlockHeightTableR::open_table(&db_read)?;
        Ok(table.get_last_block_height()?)
    }

    pub fn check_act_not_exists(&self, act_id: &ActId) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_act_not_exists(act_id)
    }

    pub fn check_acts_not_exist(&self, ids: &[ActId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_acts_not_exist(ids)
    }

    pub fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        let db_read = self.database.begin_read()?;
        let table = BlockTableR::open_table(&db_read)?;
        table.get_block(height)
    }
}

// write
impl BlockChainDbInner {
    // complete all operations within a single transaction
    pub fn commit_block(
        &self,
        height: BlockHeight,
        hash: BlockHash,
        state: BlockState,
        act_ids: Vec<ActId>,
        block: &Block,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        // update last_block_height
        {
            let mut table_lbh = LastBlockHeightTableW::open_table(&db_write)?;
            table_lbh.insert((), height)?;
        }

        // insert acts
        {
            let mut table_act = ActTableW::open_table(&db_write)?;
            for act in &block.acts {
                table_act.insert_act(&hash, &act)?;
            }
        }

        // insert block
        {
            let mut table_block = BlockTableW::open_table(&db_write)?;
            table_block.insert_block(
                height,
                &BlockInDb {
                    hash,
                    state,
                    timestamp: block.timestamp,
                    act_ids,
                },
            )?;
        }

        db_write.commit()?;
        Ok(())
    }
}
