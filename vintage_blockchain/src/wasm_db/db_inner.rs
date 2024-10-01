use crate::wasm_db::{DownloadWasmTableR, DownloadWasmTableW, WasmBinaryTableR, WasmBinaryTableW};
use redb::Database;
use std::path::Path;
use vintage_msg::WasmHash;

pub(crate) struct WasmDbInner {
    database: Database,
}

// create
impl WasmDbInner {
    pub fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = Self {
            database: Database::create(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn create_tables(&self) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        WasmBinaryTableW::open_table(&db_write)?;
        DownloadWasmTableW::open_table(&db_write)?;
        db_write.commit()?;
        Ok(())
    }
}

// read
impl WasmDbInner {
    pub fn wasm_binary_exists(&self, wasm_hash: &WasmHash) -> anyhow::Result<bool> {
        let db_read = self.database.begin_read()?;
        let table = WasmBinaryTableR::open_table(&db_read)?;
        Ok(table.wasm_binary_exists(wasm_hash)?)
    }

    pub fn get_wasm_binary(&self, wasm_hash: &WasmHash) -> anyhow::Result<Vec<u8>> {
        let db_read = self.database.begin_read()?;
        let table = WasmBinaryTableR::open_table(&db_read)?;
        Ok(table.get_wasm_binary(wasm_hash)?)
    }
}

// write
impl WasmDbInner {
    pub fn try_insert_wasm_binary(
        &self,
        wasm_hash: &WasmHash,
        wasm_binary: &[u8],
    ) -> anyhow::Result<bool> {
        let db_write = self.database.begin_write()?;
        let insert = {
            let mut table = WasmBinaryTableW::open_table(&db_write)?;
            if !table.wasm_binary_exists(wasm_hash)? {
                table.insert_wasm_binary(wasm_hash, wasm_binary)?;
                true
            } else {
                false
            }
        };

        db_write.commit()?;
        Ok(insert)
    }

    pub fn get_download_wasm_tasks(&self) -> anyhow::Result<Vec<WasmHash>> {
        let db_read = self.database.begin_read()?;
        let table = DownloadWasmTableR::open_table(&db_read)?;
        Ok(table.get_download_wasm_tasks()?)
    }

    pub fn try_insert_download_wasm_task(&self, wasm_hash: &WasmHash) -> anyhow::Result<bool> {
        let db_write = self.database.begin_write()?;
        let exits = {
            let table = WasmBinaryTableW::open_table(&db_write)?;
            table.wasm_binary_exists(wasm_hash)?
        };
        let insert = if !exits {
            let mut table = DownloadWasmTableW::open_table(&db_write)?;
            table.insert_download_wasm_task(wasm_hash)?;
            true
        } else {
            false
        };
        db_write.commit()?;
        Ok(insert)
    }

    pub fn finish_download_wasm_task(
        &self,
        wasm_hash: &WasmHash,
        wasm_binary: &[u8],
    ) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;
        {
            let mut table = WasmBinaryTableW::open_table(&db_write)?;
            table.insert_wasm_binary(wasm_hash, wasm_binary)?;
        }
        {
            let mut table = DownloadWasmTableW::open_table(&db_write)?;
            table.remove_download_wasm_task(wasm_hash)?;
        }
        db_write.commit()?;
        Ok(())
    }
}
