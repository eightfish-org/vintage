use crate::constants::{
    ACTION_BLOCK_HEIGHT, ACTION_UPDATE_INDEX, ACTION_UPGRADE_WASM, ACTION_UPLOAD_WASM,
};
use crate::VIN_2_WORKER;
use crate::{req_payload_json, InputOutputObject};
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json::json;
use tokio::sync::mpsc;
use vintage_msg::{
    ActEvent, BlockHash, BlockHeight, MsgToProxy, Proto, UpdateEntityEvent, UpgradeWasmEvent,
    UploadWasmEvent,
};
use vintage_utils::{Service, Timestamp};

pub struct Vin2Worker {
    redis_conn: Connection,
    msg_receiver: mpsc::Receiver<MsgToProxy>,
}

impl Vin2Worker {
    pub(crate) fn new(redis_conn: Connection, msg_receiver: mpsc::Receiver<MsgToProxy>) -> Self {
        Self {
            redis_conn,
            msg_receiver,
        }
    }
}

#[async_trait]
impl Service for Vin2Worker {
    type Input = ();
    type Output = ();

    async fn service(mut self, _input: Self::Input) -> Self::Output {
        loop {
            match self.msg_receiver.recv().await {
                Some(msg) => match msg {
                    MsgToProxy::BlockEvent(block_event) => {
                        self.on_block_height_event(block_event.height, &block_event.block_hash)
                            .await;
                        for ue_event in block_event.ue_events {
                            self.on_ue_event(ue_event).await;
                        }
                        for act_event in block_event.act_events {
                            self.on_act_event(
                                block_event.height,
                                &block_event.block_hash,
                                block_event.timestamp,
                                act_event,
                            )
                            .await;
                        }
                        for upgrade_wasm_events in block_event.upgrade_wasm_events {
                            self.on_upgrade_wasm_event(block_event.height, upgrade_wasm_events)
                                .await;
                        }
                    }
                    MsgToProxy::UploadWasmEvent(event) => {
                        self.on_upload_wasm_event(event).await;
                    }
                },
                None => {
                    break;
                }
            }
        }
    }
}

impl Vin2Worker {
    async fn on_block_height_event(&mut self, height: BlockHeight, block_hash: &BlockHash) {
        let playload = serde_json::to_vec(&json!({
            "block_height": height,
            "block_hash": block_hash,
        }))
        .unwrap();

        let output = InputOutputObject {
            action: ACTION_BLOCK_HEIGHT.to_owned(),
            proto: "".to_owned(),
            model: "".to_owned(),
            data: playload,
            ext: vec![],
        };

        self.publish_vin_2_worker(None, &output).await;
    }

    async fn on_ue_event(&mut self, event: UpdateEntityEvent) {
        let payload = req_payload_json(&event.req_id, &event.entity_keys);

        let proto = event.proto.clone();
        let output = InputOutputObject {
            action: ACTION_UPDATE_INDEX.to_owned(),
            proto: event.proto,
            model: "".to_owned(),
            data: payload.to_string().as_bytes().to_vec(),
            ext: vec![],
        };

        self.publish_vin_2_worker(Some(&proto), &output).await;
    }

    async fn on_act_event(
        &mut self,
        height: BlockHeight,
        block_hash: &BlockHash,
        timestamp: Timestamp,
        event: ActEvent,
    ) {
        let ext = json!({
            "block_height": height,
            "block_hash": block_hash,
            "time": timestamp,
            "nonce": event.act_number,
            "randomvec": event.random,
        });

        let proto = event.act_tx.proto.clone();
        let output = InputOutputObject {
            action: event.act_tx.action,
            proto: event.act_tx.proto,
            model: event.act_tx.model,
            data: event.act_tx.data,
            ext: ext.to_string().as_bytes().to_vec(),
        };

        self.publish_vin_2_worker(Some(&proto), &output).await;
    }

    async fn on_upload_wasm_event(&mut self, event: UploadWasmEvent) {
        log::info!(
            "upload wasm event to worker, hash: {}, size: {}B",
            event.wasm_hash,
            event.wasm_binary.len()
        );

        let output = InputOutputObject {
            action: ACTION_UPLOAD_WASM.to_string(),
            proto: "".to_owned(),
            model: "".to_owned(),
            data: event.wasm_hash.as_bytes().into(),
            ext: event.wasm_binary,
        };
        self.publish_vin_2_worker(None, &output).await;
    }

    async fn on_upgrade_wasm_event(&mut self, block_height: BlockHeight, event: UpgradeWasmEvent) {
        log::info!(
            "upgrade wasm event to worker, height: {}, proto: {}, hash: {}",
            block_height,
            event.proto,
            event.wasm_hash
        );

        let output = InputOutputObject {
            action: ACTION_UPGRADE_WASM.to_string(),
            proto: event.proto,
            model: "".to_owned(),
            data: event.wasm_hash.as_bytes().into(),
            ext: vec![],
        };
        self.publish_vin_2_worker(None, &output).await;
    }

    async fn publish_vin_2_worker(&mut self, proto: Option<&Proto>, output: &InputOutputObject) {
        let channel = match proto {
            Some(value) => format!("{}:{}", VIN_2_WORKER, value),
            None => VIN_2_WORKER.to_owned(),
        };
        let output_bytes = serde_json::to_vec(output).unwrap();

        let result: Result<u32, redis::RedisError> =
            self.redis_conn.publish(channel, output_bytes).await;
        if let Err(err) = result {
            log::error!("Error publishing to redis: {:?}", err);
        }
    }
}
