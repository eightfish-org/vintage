use crate::wasm_db::WasmDbInner;
use std::path::Path;
use std::sync::Arc;
use tokio::task::spawn_blocking;
use vintage_utils::Hashed;

#[derive(Clone)]
pub(crate) struct WasmDb {
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
    pub async fn wasm_binary_exists(&self, wasm_hash: Hashed) -> anyhow::Result<bool> {
        let db = self.db.clone();
        spawn_blocking(move || db.wasm_binary_exists(&wasm_hash)).await?
    }

    pub async fn get_wasm_binary(&self, wasm_hash: Hashed) -> anyhow::Result<Vec<u8>> {
        let db = self.db.clone();
        spawn_blocking(move || db.get_wasm_binary(&wasm_hash)).await?
    }
}

// write
impl WasmDb {
    pub async fn insert_wasm_binary(
        &self,
        wasm_hash: Hashed,
        wasm_binary: Vec<u8>,
    ) -> anyhow::Result<()> {
        let db = self.db.clone();
        spawn_blocking(move || db.insert_wasm_binary(&wasm_hash, &wasm_binary)).await?
    }
}
