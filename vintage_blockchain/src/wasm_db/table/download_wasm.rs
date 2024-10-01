use redb::ReadableTable;
use vintage_msg::WasmHash;
use vintage_utils::{define_redb_table, RedbBytes32};

define_redb_table! {
    pub(crate) (DownloadWasmTable, DownloadWasmTableR, DownloadWasmTableW) = (RedbBytes32, (), "download_wasm")
}

impl<TABLE> DownloadWasmTable<TABLE>
where
    TABLE: ReadableTable<RedbBytes32, ()>,
{
    pub fn get_download_wasm_tasks(&self) -> anyhow::Result<Vec<WasmHash>> {
        let mut iter = self.table.iter()?;
        let mut tasks = Vec::new();

        loop {
            match iter.next() {
                Some(result) => {
                    let (access, _) = result?;
                    tasks.push(access.value().into());
                }
                None => {
                    break;
                }
            }
        }

        Ok(tasks)
    }
}

impl<'db, 'txn> DownloadWasmTableW<'db, 'txn> {
    pub fn insert_download_wasm_task(&mut self, wasm_hash: &WasmHash) -> anyhow::Result<()> {
        self.insert(wasm_hash.as_bytes(), ())?;
        Ok(())
    }

    pub fn remove_download_wasm_task(&mut self, wasm_hash: &WasmHash) -> anyhow::Result<()> {
        self.table.remove(wasm_hash.as_bytes())?;
        Ok(())
    }
}
