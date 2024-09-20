use crate::constants::{ACTION_UPLOAD_WASM, ADMIN_2_VIN};
use crate::io_object::{read_msg, InputOutputObject};
use async_trait::async_trait;
use redis::aio::PubSub;
use tokio::sync::mpsc;
use vintage_msg::{MsgToBlockChain, UploadWasm};
use vintage_utils::{SendMsg, Service};

pub struct Admin2Vin {
    blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
}

impl Admin2Vin {
    pub(crate) fn new(blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>) -> Self {
        Self {
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
                self.upload_wasm(msg_obj);
            }
        }
    }
}

impl Admin2Vin {
    fn upload_wasm(&self, object: InputOutputObject) {
        self.blockchain_msg_sender
            .send_msg(MsgToBlockChain::UploadWasm(UploadWasm {
                proto: object.proto,
                wasm_binary: object.data,
                block_interval: 1000,
            }));
    }
}
