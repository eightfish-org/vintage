use crate::chain::ArcBlockChainCore;
use async_trait::async_trait;
use std::error::Error;
use vintage_consensus::BlockConsensus;
use vintage_msg::{Block, BlockHeight, Hash};

pub struct BlockConsensusImpl {
    blockchain: ArcBlockChainCore,
}

impl BlockConsensusImpl {
    pub(crate) fn new(blockchain: ArcBlockChainCore) -> Self {
        Self { blockchain }
    }
}

#[async_trait]
impl BlockConsensus<Block> for BlockConsensusImpl {
    async fn get_block_height(&self) -> Result<BlockHeight, Box<dyn Error + Send>> {
        self.blockchain.lock().await.get_block_height().await
    }

    async fn get_block(&self, height: u64) -> Result<(Block, Hash), Box<dyn Error + Send>> {
        self.blockchain.lock().await.get_block(height).await
    }

    async fn check_block(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        self.blockchain
            .lock()
            .await
            .check_block(height, block, hash)
            .await
    }

    async fn commit(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        self.blockchain
            .lock()
            .await
            .commit(height, block, hash)
            .await
    }
}
