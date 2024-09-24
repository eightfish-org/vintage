use crate::wasm_db::WasmDbInner;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_msg::WasmHash;

#[derive(Clone)]
pub struct WasmDb {
    db: Arc<WasmDbInner>,
}

// create
pub(crate) async fn create_wasm_db_inner(
    path: impl AsRef<Path> + Send + 'static,
) -> anyhow::Result<Arc<WasmDbInner>> {
    let db = spawn_blocking(|| WasmDbInner::create(path)).await??;
    Ok(Arc::new(db))
}

impl WasmDb {
    pub(crate) fn new(db: Arc<WasmDbInner>) -> Self {
        Self { db }
    }
}

// read
impl WasmDb {
    pub async fn wasm_binary_exists(&self, wasm_hash: WasmHash) -> anyhow::Result<bool> {
        let db = self.db.clone();
        spawn_blocking(move || db.wasm_binary_exists(&wasm_hash)).await?
    }

    pub async fn get_wasm_binary(&self, wasm_hash: WasmHash) -> anyhow::Result<Vec<u8>> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_wasm_binary(&wasm_hash)).await?
    }
}

// write
impl WasmDb {
    pub async fn try_insert_wasm_binary(
        &self,
        wasm_hash: WasmHash,
        wasm_binary: Vec<u8>,
    ) -> anyhow::Result<bool> {
        let db = self.db.clone();
        spawn_blocking(move || db.try_insert_wasm_binary(&wasm_hash, &wasm_binary)).await?
    }

    pub async fn try_insert_download_wasm_task(&self, wasm_hash: WasmHash) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.try_insert_download_wasm_task(&wasm_hash)).await?
    }

    pub async fn get_download_wasm_tasks(&self) -> anyhow::Result<Vec<WasmHash>> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_download_wasm_tasks()).await?
    }

    pub async fn finish_download_wasm_task(
        &self,
        wasm_hash: WasmHash,
        wasm_binary: Vec<u8>,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.finish_download_wasm_task(&wasm_hash, &wasm_binary)).await?
    }
}
