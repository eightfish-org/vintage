use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{Tx, TxContent, TxId};
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (Txs, TxsW) = (TxId, TxContent, "txs")
}

impl<TABLE> Txs<TABLE>
where
    TABLE: ReadableTable<TxId, TxContent>,
{
    pub fn check_tx_not_exists(&self, id: TxId) -> anyhow::Result<()> {
        if self.exists(id)? {
            Err(anyhow!("tx {} already exists id db", id))
        } else {
            Ok(())
        }
    }

    pub fn check_all_txs_not_exist(&self, txs: &Vec<Tx>) -> anyhow::Result<()> {
        for tx in txs {
            self.check_tx_not_exists(tx.id)?;
        }
        Ok(())
    }
}
