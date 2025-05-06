use redb::ReadableTable;
use vintage_msg::{BlockHeight, TxId};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (UpgradeWasmTable, UpgradeWasmTableR, UpgradeWasmTableW) = (BlockHeight, RedbBytes, "upgrade_wasm")
}

impl<TABLE> UpgradeWasmTable<TABLE>
where
    TABLE: ReadableTable<BlockHeight, RedbBytes>,
{
    pub fn get_upgrade_wasm_tx_ids(&self, block_height: BlockHeight) -> anyhow::Result<Vec<TxId>> {
        match self.get(block_height)? {
            Some(access) => {
                let (wasm_tx_ids, _bytes_read) = Vec::<TxId>::bincode_deserialize(access.value())?;
                Ok(wasm_tx_ids)
            }
            None => Ok(Vec::new()),
        }
    }
}

impl<'db, 'txn> UpgradeWasmTableW<'db, 'txn> {
    pub fn insert_upgrade_wasm_tx_ids(
        &mut self,
        block_height: BlockHeight,
        wasm_tx_ids: Vec<TxId>,
    ) -> anyhow::Result<()> {
        let bytes = wasm_tx_ids.bincode_serialize()?;
        self.table.insert(block_height, bytes.as_slice())?;
        Ok(())
    }
}
