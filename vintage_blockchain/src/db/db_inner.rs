use crate::chain::BlockState;
use crate::db::{
    ActTableR, ActTableW, BlockInDb, BlockTableR, BlockTableW, EntityTableR, EntityTableW,
    LastBlockHeightTableR, LastBlockHeightTableW, UpdateEntityTxPoolTableR,
    UpdateEntityTxPoolTableW, UpdateEntityTxTableR, UpdateEntityTxTableW,
};
use crate::tx::TxId;
use redb::Database;
use std::path::Path;
use vintage_msg::{Block, BlockHash, BlockHeight, EntityHash, EntityId, Model, UpdateEntityTx};

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
        LastBlockHeightTableW::open_table(&db_write)?;
        BlockTableW::open_table(&db_write)?;
        ActTableW::open_table(&db_write)?;
        UpdateEntityTxTableW::open_table(&db_write)?;
        UpdateEntityTxPoolTableW::open_table(&db_write)?;
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

    pub fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        let db_read = self.database.begin_read()?;
        let table = BlockTableR::open_table(&db_read)?;
        table.get_block(height)
    }

    pub fn check_act_not_exists(&self, act_id: &TxId) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_tx_not_exists(act_id)
    }

    pub fn check_acts_not_exist(&self, act_ids: &[TxId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTableR::open_table(&db_read)?;
        table.check_txs_not_exist(act_ids)
    }

    pub fn check_ue_txs_not_exist(&self, tx_ids: &[TxId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = UpdateEntityTxTableR::open_table(&db_read)?;
        table.check_txs_not_exist(tx_ids)
    }

    pub fn ue_tx_exists_in_pool(&self, tx_id: &TxId) -> anyhow::Result<bool> {
        let db_read = self.database.begin_read()?;
        let table = UpdateEntityTxPoolTableR::open_table(&db_read)?;
        let exists = table.exists(tx_id.as_bytes())?;
        Ok(exists)
    }

    pub fn get_ue_txs_in_pool(
        &self,
        count: usize,
    ) -> anyhow::Result<(Vec<TxId>, Vec<UpdateEntityTx>)> {
        let db_read = self.database.begin_read()?;
        let table = UpdateEntityTxPoolTableR::open_table(&db_read)?;
        table.get_ue_txs_in_pool(count)
    }

    pub fn get_entity(&self, model: &Model, entity_id: &EntityId) -> anyhow::Result<EntityHash> {
        let db_read = self.database.begin_read()?;
        let table = EntityTableR::open_table(&db_read)?;
        table.get_entity(model, entity_id)
    }
}

// write
impl BlockChainDbInner {
    pub fn insert_ue_tx_to_pool(&self, tx_id: &TxId, tx: &UpdateEntityTx) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        {
            let mut table_pool = UpdateEntityTxPoolTableW::open_table(&db_write)?;
            table_pool.check_tx_not_exists(&tx_id)?;
            let table_tx = UpdateEntityTxTableW::open_table(&db_write)?;
            table_tx.check_tx_not_exists(&tx_id)?;
            table_pool.insert_tx(&tx_id, &tx)?;
        }

        db_write.commit()?;
        Ok(())
    }

    // complete all operations within a single transaction
    pub fn commit_block(
        &self,
        height: BlockHeight,
        hash: BlockHash,
        state: BlockState,
        act_ids: Vec<TxId>,
        ue_tx_ids: Vec<TxId>,
        block: &Block,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        // remove txs in pool
        {
            let mut table_pool = UpdateEntityTxPoolTableW::open_table(&db_write)?;
            table_pool.remove_ue_txs_in_pool(&ue_tx_ids)?
        }
        // insert txs
        {
            let mut table_act = ActTableW::open_table(&db_write)?;
            for act in &block.acts {
                table_act.insert_tx(&hash, &act)?;
            }
        }
        {
            let mut table_ue_tx = UpdateEntityTxTableW::open_table(&db_write)?;
            let mut table_entity = EntityTableW::open_table(&db_write)?;
            for ue_tx in &block.ue_txs {
                table_ue_tx.insert_tx(&hash, &ue_tx)?;
                for entity in &ue_tx.entities {
                    table_entity
                        .insert_entity(&ue_tx.model, &entity.id, &entity.hash)
                        .unwrap()
                }
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
                    ue_tx_ids,
                },
            )?;
        }

        // update last_block_height
        {
            let mut table_lbh = LastBlockHeightTableW::open_table(&db_write)?;
            table_lbh.insert((), height)?;
        }

        db_write.commit()?;
        Ok(())
    }
}
