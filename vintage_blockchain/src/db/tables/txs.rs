use crate::db::DB;
use anyhow::anyhow;
use redb::{ReadableTable, StorageError};
use vintage_msg::{TxContent, TxId};
use vintage_utils::define_redb_table;

define_redb_table!(TxId, TxContent, "txs");

impl DB {
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
        let exist = DB::tx_exists(table, id)?;
        if exist {
            Err(anyhow!("tx {} already exists id db", id))
        } else {
            Ok(())
        }
    }
}
