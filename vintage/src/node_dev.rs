use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use vintage_blockchain::{BlockChain, BlockChainApiImpl};
use vintage_consensus::BlockConsensus;
use vintage_msg::{BlockChainApi, BlockHeight};
use vintage_utils::Service;

pub struct VintageNodeDev {
    block_consensus: BlockChain,
    next_height: BlockHeight,
}

impl VintageNodeDev {
    pub async fn create(
        block_consensus: BlockChain,
        blockchain_api: Arc<BlockChainApiImpl>,
    ) -> anyhow::Result<Self> {
        let next_height = blockchain_api.get_block_height().await? + 1;
        Ok(Self {
            block_consensus,
            next_height,
        })
    }
}

#[async_trait]
impl Service for VintageNodeDev {
    type Input = ();
    type Output = ();

    async fn service(mut self, _input: Self::Input) -> Self::Output {
        loop {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            self.run().await
        }
    }
}

impl VintageNodeDev {
    async fn run(&mut self) {
        let (block, hash) = match self.block_consensus.get_block(self.next_height).await {
            Ok(value) => {
                value
            }
            Err(err) => {
                log::error!("get block err: {:?}", err);
                return
            }
        };
        match self.block_consensus.check_block(self.next_height, block.clone(), hash.clone()).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("check block err: {:?}", err);
                return
            }
        }
        match self.block_consensus.commit(self.next_height, block, hash).await {
            Ok(_) => {}
            Err(err) => {
                log::error!("commit block err: {:?}", err);
                return
            }
        }

        self.next_height += 1;
    }
}
