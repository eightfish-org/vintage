use crate::constants::{ACTION_UPLOAD_WASM, ADMIN_2_VIN};
use crate::io_object::{read_msg, InputOutputObject};
use async_trait::async_trait;
use redis::aio::PubSub;
use serde::Deserialize;
use std::cmp::max;
use tokio::sync::mpsc;
use vintage_msg::{MsgToBlockChain, UploadWasm};
use vintage_utils::{SendMsg, Service};

pub struct Admin2Vin {
    min_after_blocks: u64,
    blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
}

impl Admin2Vin {
    pub(crate) fn new(
        min_after_blocks: u64,
        blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
    ) -> Self {
        Self {
            min_after_blocks,
            blockchain_msg_sender,
        }
    }
}

#[async_trait]
impl Service for Admin2Vin {
    type Input = PubSub;
    type Output = anyhow::Result<()>;

    async fn service(mut self, mut pubsub: Self::Input) -> Self::Output {
        pubsub.subscribe(ADMIN_2_VIN).await?;
        let mut pubsub_stream = pubsub.on_message();

        loop {
            let msg_obj = read_msg(&mut pubsub_stream, ADMIN_2_VIN).await?;

            if &msg_obj.action == ACTION_UPLOAD_WASM {
                if let Err(err) = self.upload_wasm(msg_obj) {
                    log::error!("upload wasm err: {:?}", err);
                }
            }
        }
    }
}

impl Admin2Vin {
    fn upload_wasm(&self, object: InputOutputObject) -> anyhow::Result<()> {
        #[allow(dead_code)]
        #[derive(Deserialize)]
        struct UploadWasmReq {
            // proto: Proto,
            // version: String,
            // digest: String,
            // timestamp: Timestamp,
            afterblocks: u64,
        }
        let req: UploadWasmReq = serde_json::from_str(&object.model)?;

        self.blockchain_msg_sender
            .send_msg(MsgToBlockChain::UploadWasm(UploadWasm {
                proto: object.proto,
                wasm_binary: object.ext,
                after_blocks: max(self.min_after_blocks, req.afterblocks),
            }));

        Ok(())
    }
}
