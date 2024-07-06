use crate::db::DbTable;
use anyhow::anyhow;
use redb::{ReadableTable, StorageError};
use vintage_msg::{Tx, TxContent, TxId};
use vintage_utils::define_redb_table;

define_redb_table!(TxId, TxContent, "txs");

impl DbTable {
    pub fn tx_exists<TABLE>(table: &TABLE, id: TxId) -> Result<bool, StorageError>
    where
        TABLE: ReadableTable<TxId, TxContent>,
    {
        let option = table.get(id)?;
        Ok(option.is_some())
    }

    pub fn check_tx_not_exists<TABLE>(table: &TABLE, id: TxId) -> anyhow::Result<()>
    where
        TABLE: ReadableTable<TxId, TxContent>,
    {
        let exist = DbTable::tx_exists(table, id)?;
        if exist {
            Err(anyhow!("tx {} already exists id db", id))
        } else {
            Ok(())
        }
    }

    pub fn check_all_txs_not_exist_in_db<TABLE>(table: &TABLE, txs: &Vec<Tx>) -> anyhow::Result<()>
    where
        TABLE: ReadableTable<TxId, TxContent>,
    {
        for tx in txs {
            DbTable::check_tx_not_exists(table, tx.id)?;
        }
        Ok(())
    }
}
