use crate::wasm::DownloadWasmTask;
use crate::wasm_db::WasmDb;
use async_trait::async_trait;
use vintage_utils::{Service, ServiceStarter};

pub struct DownloadWasmTasks {
    wasm_db: WasmDb,
}

impl DownloadWasmTasks {
    pub fn new(wasm_db: WasmDb) -> Self {
        Self { wasm_db }
    }
}

#[async_trait]
impl Service for DownloadWasmTasks {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        match self.wasm_db.get_download_wasm_tasks().await {
            Ok(tasks) => {
                for wasm_hash in tasks {
                    ServiceStarter::new(DownloadWasmTask::new(wasm_hash)).start();
                }
            }
            Err(err) => {
                log::error!("get_download_wasm_tasks err: {:?}", err)
            }
        }
    }
}
