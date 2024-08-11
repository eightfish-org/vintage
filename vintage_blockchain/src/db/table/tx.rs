use vintage_msg::{Act, UpdateEntityTx};

macro_rules! define_tx_table {
    ($vis:vis ($table:ident, $table_r:ident, $table_w:ident) = ($tx:ty, $table_name:literal)) => {
        vintage_utils::define_redb_table! {
            $vis ($table, $table_r, $table_w) = (vintage_utils::RedbBytes32, vintage_utils::RedbBytes, $table_name)
        }

        impl<TABLE> $table<TABLE>
        where
            TABLE: redb::ReadableTable<vintage_utils::RedbBytes32, vintage_utils::RedbBytes>,
        {
            pub fn check_tx_not_exists(&self, tx_id: &$crate::tx::TxId) -> anyhow::Result<()> {
                if self.exists(tx_id.as_bytes())? {
                    Err(anyhow::anyhow!("{} {} already exists id db", $table_name, tx_id))
                } else {
                    Ok(())
                }
            }

            #[allow(dead_code)]
            pub fn check_txs_not_exist(&self, tx_ids: &[$crate::tx::TxId]) -> anyhow::Result<()> {
                for id in tx_ids {
                    self.check_tx_not_exists(id)?;
                }
                Ok(())
            }
        }

        impl<'db, 'txn> $table_w<'db, 'txn> {
            pub fn insert_tx(&mut self, tx_id: &$crate::tx::TxId, tx: &$tx) -> anyhow::Result<()> {
                let bytes = vintage_utils::BincodeSerialize::bincode_serialize(tx)?;
                self.insert(tx_id.as_bytes(), &*bytes)?;
                Ok(())
            }
        }
    }
}

define_tx_table! {
    pub(crate) (ActTable, ActTableR, ActTableW) = (Act, "act")
}
define_tx_table! {
    pub(crate) (UpdateEntityTxTable, UpdateEntityTxTableR, UpdateEntityTxTableW)= (UpdateEntityTx, "update_entity_tx")
}
define_tx_table! {
    pub(crate) (UpdateEntityTxPoolTable, UpdateEntityTxPoolTableR, UpdateEntityTxPoolTableW) = (UpdateEntityTx, "update_entity_tx_pool")
}