use anyhow::anyhow;
use async_trait::async_trait;
use std::error::Error;
use std::time::Duration;
use vintage_blockchain::BlockConsensusImpl;
use vintage_consensus::BlockConsensus;
use vintage_msg::BlockHeight;
use vintage_utils::{Service, ServiceStarter};

pub struct VintageNodeDev {
    block_consensus: BlockConsensusImpl,
    block_interval: u64,
    next_height: BlockHeight,
}

impl VintageNodeDev {
    pub async fn create(
        block_interval: u64,
        block_consensus: BlockConsensusImpl,
    ) -> anyhow::Result<ServiceStarter<Self>> {
        let block_height = block_consensus
            .get_block_height()
            .await
            .map_err(|err| anyhow!("get_block_height err: {:?}", err))?;
        Ok(ServiceStarter::new(Self {
            block_consensus,
            block_interval,
            next_height: block_height + 1,
        }))
    }
}

#[async_trait]
impl Service for VintageNodeDev {
    type Input = ();
    type Output = ();

    async fn service(mut self, _input: Self::Input) -> Self::Output {
        loop {
            tokio::time::sleep(Duration::from_millis(self.block_interval)).await;
            if let Err(err) = self.generate_block().await {
                log::error!("generate_block err: {:?}", err);
            }
        }
    }
}

impl VintageNodeDev {
    async fn generate_block(&mut self) -> Result<(), Box<dyn Error + Send>> {
        let (block, hash) = self.block_consensus.new_block(self.next_height).await?;
        self.block_consensus
            .check_block(self.next_height, block.clone(), hash.clone())
            .await?;
        self.block_consensus
            .commit_block(self.next_height, block, hash)
            .await?;
        self.next_height += 1;
        Ok(())
    }
}
