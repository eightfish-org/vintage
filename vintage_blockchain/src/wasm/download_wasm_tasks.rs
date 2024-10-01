use crate::network::BlockChainNetworkClient;
use crate::proxy::MsgToProxySender;
use crate::wasm::DownloadWasmTask;
use crate::wasm_db::WasmDb;
use async_trait::async_trait;
use std::sync::Arc;
use vintage_utils::{Service, ServiceStarter};

pub struct DownloadWasmTasks {
    wasm_db: WasmDb,
    proxy_msg_sender: MsgToProxySender,
    client: Arc<BlockChainNetworkClient>,
}

impl DownloadWasmTasks {
    pub(crate) fn new(
        wasm_db: WasmDb,
        proxy_msg_sender: MsgToProxySender,
        client: Arc<BlockChainNetworkClient>,
    ) -> Self {
        Self {
            wasm_db,
            proxy_msg_sender,
            client,
        }
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
                    ServiceStarter::new(DownloadWasmTask::new(
                        self.wasm_db.clone(),
                        self.proxy_msg_sender.clone(),
                        self.client.clone(),
                        wasm_hash,
                    ))
                    .start();
                }
            }
            Err(err) => {
                log::error!("get_download_wasm_tasks err: {:?}", err)
            }
        }
    }
}
