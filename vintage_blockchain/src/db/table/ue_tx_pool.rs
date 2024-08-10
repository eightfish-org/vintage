use crate::db::{UpdateEntityTxPoolTable, UpdateEntityTxPoolTableW};
use crate::tx::TxId;
use redb::ReadableTable;
use vintage_msg::UpdateEntityTx;
use vintage_utils::{BincodeDeserialize, RedbBytes, RedbBytes32};

impl<TABLE> UpdateEntityTxPoolTable<TABLE>
where
    TABLE: ReadableTable<RedbBytes32, RedbBytes>,
{
    pub fn get_ue_txs_in_pool(
        &self,
        mut count: usize,
    ) -> anyhow::Result<(Vec<TxId>, Vec<UpdateEntityTx>)> {
        let mut iter = self.table.iter()?;
        let mut tx_ids = Vec::new();
        let mut txs = Vec::new();

        loop {
            if count == 0 {
                break;
            }
            let (id, tx) = match iter.next() {
                Some(result) => result?,
                None => {
                    break;
                }
            };

            let (tx, _bytes_read) = UpdateEntityTx::bincode_deserialize(tx.value())?;
            tx_ids.push(TxId::from(id.value()));
            txs.push(tx);

            count -= 1;
        }

        Ok((tx_ids, txs))
    }
}

impl<'db, 'txn> UpdateEntityTxPoolTableW<'db, 'txn> {
    pub fn remove_ue_txs_in_pool(&mut self, tx_ids: &[TxId]) -> anyhow::Result<()> {
        for tx_id in tx_ids {
            self.table.remove(tx_id.as_bytes())?;
        }
        Ok(())
    }
}
