use async_trait::async_trait;
use vintage_utils::Service;

pub struct BlockSyncService {}

impl BlockSyncService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Service for BlockSyncService {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {}
}
