use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{WasmId, WasmInfo};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (WasmTxTable, WasmTxTableR, WasmTxTableW) = (RedbBytes, RedbBytes, "wasm_tx")
}

impl<TABLE> WasmTxTable<TABLE>
where
    TABLE: ReadableTable<RedbBytes, RedbBytes>,
{
    pub fn check_wasm_tx_not_exists(&self, wasm_id: &WasmId) -> anyhow::Result<()> {
        let id = wasm_id.bincode_serialize()?;
        if self.exists(id.as_slice())? {
            Err(anyhow::anyhow!(
                "wasm tx {} {} already exists id db",
                wasm_id.proto,
                wasm_id.wasm_hash
            ))
        } else {
            Ok(())
        }
    }

    pub fn check_wasm_txs_not_exist(&self, wasm_ids: &[WasmId]) -> anyhow::Result<()> {
        for wasm_id in wasm_ids {
            self.check_wasm_tx_not_exists(wasm_id)?;
        }
        Ok(())
    }

    pub fn get_wasm_tx(&self, wasm_id: &WasmId) -> anyhow::Result<WasmInfo> {
        let id = wasm_id.bincode_serialize()?;
        match self.get(id.as_slice())? {
            Some(access) => {
                let (value, _bytes_read) = WasmInfo::bincode_deserialize(access.value())?;
                Ok(value)
            }
            None => Err(anyhow!(
                "wasm tx {} {} not found",
                wasm_id.proto,
                wasm_id.wasm_hash
            )),
        }
    }
}

impl<'db, 'txn> WasmTxTableW<'db, 'txn> {
    pub fn insert_wasm_tx(&mut self, wasm_id: &WasmId, wasm_info: &WasmInfo) -> anyhow::Result<()> {
        let id = wasm_id.bincode_serialize()?;
        let info = wasm_info.bincode_serialize()?;
        self.insert(id.as_slice(), info.as_slice())?;
        Ok(())
    }
}
