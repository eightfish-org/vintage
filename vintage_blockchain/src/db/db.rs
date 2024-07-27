use crate::db::{
    ActTableR, ActTableW, BlockInDb, BlockTableR, BlockTableW, LastBlockHeightTableR,
    LastBlockHeightTableW,
};
use redb::Database;
use std::path::Path;
use vintage_msg::{ActId, Block, BlockHash, BlockHeight};

pub(crate) struct BlockChainDb {
    database: Database,
}

// create
impl BlockChainDb {
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
impl BlockChainDb {
    pub fn check_act_not_exists(&self, id: ActId) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_act_not_exists(id)
    }

    pub fn check_acts_not_exist(&self, ids: &[ActId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_acts_not_exist(ids)
    }

    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db_read = self.database.begin_read()?;
        let table = LastBlockHeightTableR::open_table(&db_read)?;
        Ok(table.get_last_block_height()?)
    }

    pub fn get_block_hash(&self, block_height: BlockHeight) -> anyhow::Result<BlockHash> {
        let db_read = self.database.begin_read()?;
        let table = BlockTableR::open_table(&db_read)?;
        Ok(table.get_block(block_height)?.hash)
    }
}

// write
impl BlockChainDb {
    // complete all operations within a single transaction
    pub fn write_block(&self, block: &Block) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        // update last_block_height
        {
            let mut table_lbh = LastBlockHeightTableW::open_table(&db_write)?;
            table_lbh.insert((), block.header.height)?;
        }

        let mut act_ids = Vec::<ActId>::new();
        // insert acts
        {
            let mut table_act = ActTableW::open_table(&db_write)?;
            for act in &block.body.acts {
                act_ids.push(act.id);
                table_act.insert(act.id, &act.content)?;
            }
        }

        // insert block
        {
            let mut table_block = BlockTableW::open_table(&db_write)?;
            table_block.insert_block(
                block.header.height,
                &BlockInDb {
                    hash: block.header.hash.clone(),
                    timestamp: block.header.timestamp,
                    act_ids,
                },
            )?;
        }

        db_write.commit()?;
        Ok(())
    }
}
