use crate::network::BlockChainNetworkClient;
use crate::proxy::MsgToProxySender;
use crate::wasm_db::WasmDb;
use async_trait::async_trait;
use std::sync::Arc;
use vintage_msg::WasmHash;
use vintage_utils::Service;

pub(crate) struct DownloadWasmTask {
    wasm_db: WasmDb,
    proxy_msg_sender: MsgToProxySender,
    client: Arc<BlockChainNetworkClient>,
    wasm_hash: WasmHash,
}

impl DownloadWasmTask {
    pub fn new(
        wasm_db: WasmDb,
        proxy_msg_sender: MsgToProxySender,
        client: Arc<BlockChainNetworkClient>,
        wasm_hash: WasmHash,
    ) -> Self {
        Self {
            wasm_db,
            proxy_msg_sender,
            client,
            wasm_hash,
        }
    }
}

#[async_trait]
impl Service for DownloadWasmTask {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        if let Err(err) = self.service_impl().await {
            log::error!("DownloadWasmTask err: {:?}", err);
        }
    }
}

impl DownloadWasmTask {
    async fn service_impl(self) -> anyhow::Result<()> {
        let (node_id, _exist) = self
            .client
            .request_wasm_exists(self.wasm_hash.clone())
            .await?;
        let wasm_binary = self
            .client
            .request_wasm(self.wasm_hash.clone(), node_id)
            .await?;
        self.wasm_db
            .finish_download_wasm_task(self.wasm_hash.clone(), wasm_binary.clone())
            .await?;
        log::info!(
            "wasm file downloaded, hash: {}, size: {}B, saved in db",
            self.wasm_hash,
            wasm_binary.len()
        );

        self.proxy_msg_sender
            .send_wasm_binary(self.wasm_hash, wasm_binary);
        Ok(())
    }
}
