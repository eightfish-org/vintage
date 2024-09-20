use crate::wasm_db::{WasmBinaryTableR, WasmBinaryTableW};
use redb::Database;
use std::path::Path;
use vintage_utils::Hashed;

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
        db_write.commit()?;
        Ok(())
    }
}

// read
impl WasmDbInner {
    pub fn wasm_binary_exists(&self, wasm_hash: &Hashed) -> anyhow::Result<bool> {
        let db_read = self.database.begin_read()?;
        let table = WasmBinaryTableR::open_table(&db_read)?;
        Ok(table.wasm_binary_exists(wasm_hash)?)
    }

    pub fn get_wasm_binary(&self, wasm_hash: &Hashed) -> anyhow::Result<Vec<u8>> {
        let db_read = self.database.begin_read()?;
        let table = WasmBinaryTableR::open_table(&db_read)?;
        Ok(table.get_wasm_binary(wasm_hash)?)
    }
}

// write
impl WasmDbInner {
    pub fn insert_wasm_binary(&self, wasm_hash: &Hashed, wasm_binary: &[u8]) -> anyhow::Result<()> {
        let db_write = self.database.begin_write()?;

        {
            let mut table = WasmBinaryTableW::open_table(&db_write)?;
            table.insert_wasm_binary(wasm_hash, wasm_binary)?;
        }

        db_write.commit()?;
        Ok(())
    }
}
