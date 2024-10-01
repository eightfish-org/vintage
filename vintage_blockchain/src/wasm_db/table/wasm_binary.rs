use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::WasmHash;
use vintage_utils::{define_redb_table, RedbBytes, RedbBytes32};

define_redb_table! {
    pub(crate) (WasmBinaryTable, WasmBinaryTableR, WasmBinaryTableW) = (RedbBytes32, RedbBytes, "wasm_binary")
}

impl<TABLE> WasmBinaryTable<TABLE>
where
    TABLE: ReadableTable<RedbBytes32, RedbBytes>,
{
    pub fn wasm_binary_exists(&self, wasm_hash: &WasmHash) -> anyhow::Result<bool> {
        Ok(self.exists(wasm_hash.as_bytes())?)
    }

    pub fn get_wasm_binary(&self, wasm_hash: &WasmHash) -> anyhow::Result<Vec<u8>> {
        match self.get(wasm_hash.as_bytes())? {
            Some(access) => Ok(access.value().into()),
            None => Err(anyhow!("wasm file {} not found", wasm_hash)),
        }
    }
}

impl<'db, 'txn> WasmBinaryTableW<'db, 'txn> {
    pub fn insert_wasm_binary(
        &mut self,
        wasm_hash: &WasmHash,
        wasm_binary: &[u8],
    ) -> anyhow::Result<()> {
        self.insert(wasm_hash.as_bytes(), wasm_binary)?;
        Ok(())
    }
}
