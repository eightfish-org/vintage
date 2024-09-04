use crate::chain::ArcBlockChainCore;
use crate::network::{
    BlockChainNetworkClient, ReqBlock, ReqBlockHash,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use std::time::Duration;
use vintage_utils::{current_timestamp, SendMsg, Service};

pub struct BlockSyncService {
    interval: u64,
    client: BlockChainNetworkClient,
    block_synced_sender: mpsc::Sender<u64>
}

impl BlockSyncService {
    pub(crate) fn new(
        block_interval: u64,
        client: BlockChainNetworkClient,
        block_synced_sender: mpsc::Sender<u64>
    ) -> Self {
        Self {
            interval: block_interval * 10,
            client,
            block_synced_sender
        }
    }
}

#[async_trait]
impl Service for BlockSyncService {
    type Input = ArcBlockChainCore;
    type Output = ();

    async fn service(mut self, blockchain_core: Self::Input) -> Self::Output {
        loop {
            log::info!("====Block sync service loop");
            tokio::time::sleep(Duration::from_millis(self.interval)).await;
            {
                let guard = blockchain_core.lock().await;
                if guard.get_last_commited_time() + self.interval > current_timestamp() {
                    continue;
                }
            }

            loop {
                match self.sync_blocks(&blockchain_core).await {
                    Ok((finished,new_height)) => {
                        if finished {
                            log::info!("====Block sync completed. notify new height: {}", new_height);
                            self.block_synced_sender.send_msg(new_height);
                            break;
                        }
                    }
                    Err(err) => {
                        log::error!("Block sync service err: {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}

impl BlockSyncService {
    const BLOCK_COUNT: u64 = 10;

    async fn sync_blocks(&mut self, blockchain_core: &ArcBlockChainCore) -> anyhow::Result<(bool, u64)> {
        log::info!("====Block sync start");

        let mut guard = blockchain_core.lock().await;

        // block height
        let block_height = guard.get_block_height().await?;
        log::info!("====Block sync block_height: {}", block_height);
        // block hash
        let rsp_block_hash = self
            .client
            .request_block_hash(ReqBlockHash {
                begin_height: block_height + 1,
                count: Self::BLOCK_COUNT,
            })
            .await?;
        let block_count = rsp_block_hash.hash_list.len() as u64;
        log::info!("====Block sync hash block_count: {}", block_count);
        // block
        let rsp_block = self
            .client
            .request_block(ReqBlock {
                begin_height: block_height + 1,
                count: block_count,
            })
            .await?;
        if rsp_block.block_list.len() != block_count as usize {
            return Err(anyhow::anyhow!("Block count mismatch"));
        }
        log::info!("====Block sync start import block count: {}", block_count);
        // import block
        for index in 0..block_count {
            guard
                .import_block(
                    block_height + index + 1,
                    rsp_block.block_list[index as usize].clone(),
                    rsp_block_hash.hash_list[index as usize].clone(),
                )
                .await?
        }
        log::info!("====Block sync imported block_count: {}", block_count);
        Ok((block_count < Self::BLOCK_COUNT, block_height + block_count))
    }
}
