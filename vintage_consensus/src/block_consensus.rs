use async_trait::async_trait;
use overlord::types::Hash;
use overlord::Codec;
use std::error::Error;

#[async_trait]
pub trait BlockConsensus<T: Codec> {
    async fn get_block(&self, height: u64) -> Result<(T, Hash), Box<dyn Error + Send>>;

    async fn check_block(
        &self,
        height: u64,
        block: T,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>>;

    async fn commit(&self, height: u64, block: T, hash: Hash) -> Result<(), Box<dyn Error + Send>>;
}
