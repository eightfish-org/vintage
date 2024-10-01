use redb::ReadableTable;
use vintage_msg::{BlockHeight, WasmId};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (UpgradeWasmTable, UpgradeWasmTableR, UpgradeWasmTableW) = (BlockHeight, RedbBytes, "upgrade_wasm")
}

impl<TABLE> UpgradeWasmTable<TABLE>
where
    TABLE: ReadableTable<BlockHeight, RedbBytes>,
{
    pub fn get_upgrade_wasm_ids(&self, block_height: BlockHeight) -> anyhow::Result<Vec<WasmId>> {
        match self.get(block_height)? {
            Some(access) => {
                let (wasm_ids, _bytes_read) = Vec::<WasmId>::bincode_deserialize(access.value())?;
                Ok(wasm_ids)
            }
            None => Ok(Vec::new()),
        }
    }
}

impl<'db, 'txn> UpgradeWasmTableW<'db, 'txn> {
    pub fn insert_upgrade_wasm_ids(
        &mut self,
        block_height: BlockHeight,
        wasm_ids: Vec<WasmId>,
    ) -> anyhow::Result<()> {
        let bytes = wasm_ids.bincode_serialize()?;
        self.table.insert(block_height, bytes.as_slice())?;
        Ok(())
    }
}
