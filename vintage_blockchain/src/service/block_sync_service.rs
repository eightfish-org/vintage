use crate::chain::ArcBlockChainCore;
use crate::network::MsgToNetworkSender;
use async_trait::async_trait;
use std::time::Duration;
use vintage_network::client::NetworkClient;
use vintage_utils::{current_timestamp, Service};

pub struct BlockSyncService {
    consensus_timeout: u64,
    client: NetworkClient,
    network_msg_sender: MsgToNetworkSender,
}

impl BlockSyncService {
    pub(crate) fn new(
        block_interval: u64,
        client: NetworkClient,
        network_msg_sender: MsgToNetworkSender,
    ) -> Self {
        Self {
            consensus_timeout: block_interval * 10,
            client,
            network_msg_sender,
        }
    }
}

#[async_trait]
impl Service for BlockSyncService {
    type Input = ArcBlockChainCore;
    type Output = ();

    async fn service(mut self, blockchain_core: Self::Input) -> Self::Output {
        loop {
            tokio::time::sleep(Duration::from_millis(self.consensus_timeout)).await;
            if let Err(err) = self.run(&blockchain_core).await {
                log::error!("Block sync service err: {:?}", err);
            }
        }
    }
}

impl BlockSyncService {
    async fn run(&mut self, blockchain_core: &ArcBlockChainCore) -> anyhow::Result<()> {
        let guard = blockchain_core.lock().await;
        if guard.get_last_commited_time() + self.consensus_timeout > current_timestamp() {
            return Ok(());
        }

        log::info!("Block sync start");

        // let height = guard.get_block_height().await?;

        Ok(())
    }
}
