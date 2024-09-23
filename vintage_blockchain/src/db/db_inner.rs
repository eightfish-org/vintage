use crate::chain::BlockState;
use crate::db::{
    ActTxTableR, ActTxTableW, BlockHeightTableR, BlockHeightTableW, BlockInDb, BlockTableR,
    BlockTableW, EntityTableR, EntityTableW, UpdateEntityTxPoolTableR, UpdateEntityTxPoolTableW,
    UpdateEntityTxTableR, UpdateEntityTxTableW, UpgradeWasmTableR, UpgradeWasmTableW, WasmTxTableR,
    WasmTxTableW,
};
use crate::tx::TxId;
use redb::Database;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::path::Path;
use vintage_msg::{
    Block, BlockHash, BlockHeight, EntityHash, EntityId, Model, UpdateEntityTx, WasmId, WasmInfo,
    WasmTx,
};

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
        BlockHeightTableW::open_table(&db_write)?;
        BlockTableW::open_table(&db_write)?;
        ActTxTableW::open_table(&db_write)?;
        UpdateEntityTxTableW::open_table(&db_write)?;
        UpdateEntityTxPoolTableW::open_table(&db_write)?;
        WasmTxTableW::open_table(&db_write)?;
        db_write.commit()?;
        Ok(())
    }
}

// read
impl BlockChainDbInner {
    pub fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        let db_read = self.database.begin_read()?;
        let table = BlockHeightTableR::open_table(&db_read)?;
        Ok(table.get_block_height()?)
    }

    pub fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        let db_read = self.database.begin_read()?;
        let table = BlockTableR::open_table(&db_read)?;
        table.get_block(height)
    }

    pub fn get_network_block(&self, height: BlockHeight) -> anyhow::Result<Block> {
        let db_read = self.database.begin_read()?;
        let block = {
            let table = BlockTableR::open_table(&db_read)?;
            table.get_block(height)?
        };
        let mut act_txs = Vec::new();
        {
            let table = ActTxTableR::open_table(&db_read)?;
            for act_tx_id in block.act_tx_ids {
                let act_tx = table.get_tx(&act_tx_id)?;
                act_txs.push(act_tx);
            }
        }
        let mut ue_txs = Vec::new();
        {
            let table = UpdateEntityTxTableR::open_table(&db_read)?;
            for ue_tx_id in block.ue_tx_ids {
                let ue_tx = table.get_tx(&ue_tx_id)?;
                ue_txs.push(ue_tx);
            }
        }
        let mut wasm_txs = Vec::new();
        {
            let table = WasmTxTableR::open_table(&db_read)?;
            for wasm_id in block.wasm_ids {
                let wasm_info = table.get_wasm_tx(&wasm_id)?;
                wasm_txs.push(WasmTx { wasm_id, wasm_info });
            }
        }
        Ok(Block {
            timestamp: block.timestamp,
            act_txs,
            ue_txs,
            wasm_txs,
        })
    }

    pub fn check_act_tx_not_exists(&self, act_tx_id: &TxId) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTxTableR::open_table(&db_read)?;
        table.check_tx_not_exists(act_tx_id)
    }

    pub fn check_act_txs_not_exist(&self, act_tx_ids: &[TxId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = ActTxTableR::open_table(&db_read)?;
        table.check_txs_not_exist(act_tx_ids)
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

    pub fn check_wasm_tx_not_exists(&self, wasm_id: &WasmId) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = WasmTxTableR::open_table(&db_read)?;
        table.check_wasm_tx_not_exists(wasm_id)
    }

    pub fn check_wasm_txs_not_exist(&self, wasm_ids: &[WasmId]) -> anyhow::Result<()> {
        let db_read = self.database.begin_read()?;
        let table = WasmTxTableR::open_table(&db_read)?;
        table.check_wasm_txs_not_exist(wasm_ids)
    }

    pub fn get_wasm_tx(&self, wasm_id: &WasmId) -> anyhow::Result<WasmInfo> {
        let db_read = self.database.begin_read()?;
        let table = WasmTxTableR::open_table(&db_read)?;
        table.get_wasm_tx(wasm_id)
    }

    pub fn get_upgrade_wasm_ids(&self, block_height: BlockHeight) -> anyhow::Result<Vec<WasmId>> {
        let db_read = self.database.begin_read()?;
        let table = UpgradeWasmTableR::open_table(&db_read)?;
        table.get_upgrade_wasm_ids(block_height)
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
        act_tx_ids: Vec<TxId>,
        ue_tx_ids: Vec<TxId>,
        wasm_ids: Vec<WasmId>,
        block: Block,
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        // remove txs in pool
        {
            let mut table_pool = UpdateEntityTxPoolTableW::open_table(&db_write)?;
            table_pool.remove_ue_txs_in_pool(&ue_tx_ids)?
        }
        // insert txs
        {
            let mut table_act_tx = ActTxTableW::open_table(&db_write)?;
            for act_tx in &block.act_txs {
                table_act_tx.insert_tx(&hash, &act_tx)?;
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
        let mut height_to_wasm_ids: HashMap<BlockHeight, Vec<WasmId>> = HashMap::new();
        {
            let mut table_wasm_tx = WasmTxTableW::open_table(&db_write)?;
            for wasm_tx in block.wasm_txs {
                table_wasm_tx.insert_wasm_tx(&wasm_tx.wasm_id, &wasm_tx.wasm_info)?;
                match height_to_wasm_ids.entry(height + wasm_tx.wasm_info.block_interval) {
                    Entry::Occupied(mut entry) => {
                        entry.get_mut().push(wasm_tx.wasm_id);
                    }
                    Entry::Vacant(entry) => {
                        let mut wasm_ids: Vec<WasmId> = Vec::new();
                        wasm_ids.push(wasm_tx.wasm_id);
                        entry.insert(wasm_ids);
                    }
                }
            }
        }
        {
            let mut table = UpgradeWasmTableW::open_table(&db_write)?;
            for (future_height, wasm_ids) in height_to_wasm_ids {
                table.insert_upgrade_wasm_ids(future_height, wasm_ids)?;
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
                    act_tx_ids,
                    ue_tx_ids,
                    wasm_ids,
                },
            )?;
        }

        // update block height
        {
            let mut table_block_height = BlockHeightTableW::open_table(&db_write)?;
            table_block_height.insert((), height)?;
        }

        db_write.commit()?;
        Ok(())
    }
}
