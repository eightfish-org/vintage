use crate::chain::ArcBlockChainCore;
use async_trait::async_trait;
use std::error::Error;
use vintage_consensus::BlockConsensus;
use vintage_msg::{Block, BlockHash, BlockHeight, Hash};

pub struct BlockConsensusImpl {
    blockchain_core: ArcBlockChainCore,
}

impl BlockConsensusImpl {
    pub(crate) fn new(blockchain_core: ArcBlockChainCore) -> Self {
        Self { blockchain_core }
    }
}

#[async_trait]
impl BlockConsensus<Block> for BlockConsensusImpl {
    async fn get_block_height(&self) -> Result<BlockHeight, Box<dyn Error + Send>> {
        let height = { self.blockchain_core.lock().await.get_block_height().await }?;
        Ok(height)
    }

    async fn new_block(&self, height: u64) -> Result<(Block, Hash), Box<dyn Error + Send>> {
        let (block, hash) = { self.blockchain_core.lock().await.new_block(height).await }?;
        Ok((block, (&hash).into()))
    }

    async fn check_block(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        let block_hash: BlockHash = (&hash).try_into()?;
        {
            self.blockchain_core
                .lock()
                .await
                .check_block(height, block, block_hash)
                .await
        }?;
        Ok(())
    }

    async fn commit_block(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        let block_hash: BlockHash = (&hash).try_into()?;
        {
            self.blockchain_core
                .lock()
                .await
                .commit_block(height, block, block_hash)
                .await
        }?;
        Ok(())
    }
}
