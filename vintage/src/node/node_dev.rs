use anyhow::anyhow;
use async_trait::async_trait;
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
            self.run().await
        }
    }
}

impl VintageNodeDev {
    async fn run(&mut self) {
        let (block, hash) = match self.block_consensus.new_block(self.next_height).await {
            Ok(value) => value,
            Err(err) => {
                log::error!("get block err: {:?}", err);
                return;
            }
        };
        match self
            .block_consensus
            .check_block(self.next_height, block.clone(), hash.clone())
            .await
        {
            Ok(_) => {}
            Err(err) => {
                log::error!("check block err: {:?}", err);
                return;
            }
        }
        match self
            .block_consensus
            .commit_block(self.next_height, block, hash)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                log::error!("commit block err: {:?}", err);
                return;
            }
        }

        self.next_height += 1;
    }
}
