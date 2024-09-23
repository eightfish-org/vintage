use async_trait::async_trait;
use vintage_utils::{Hashed, Service};

pub(crate) struct DownloadWasmTask {
    wasm_hash: Hashed,
}

impl DownloadWasmTask {
    pub fn new(wasm_hash: Hashed) -> Self {
        Self { wasm_hash }
    }
}

#[async_trait]
impl Service for DownloadWasmTask {
    type Input = ();
    type Output = ();

    async fn service(self, _input: Self::Input) -> Self::Output {
        todo!()
    }
}
