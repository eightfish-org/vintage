use crate::constants::{
    ACTION_NEW_BLOCK_HEIGHT, ACTION_UPDATE_INDEX, ACTION_UPGRADE_WASM, ACTION_UPLOAD_WASM,
};
use crate::VIN_2_WORKER;
use crate::{payload_json, InputOutputObject};
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json::json;
use tokio::sync::mpsc;
use vintage_msg::{ActEvent, BlockHeight, MsgToProxy, Proto, UpdateEntityEvent, WasmHash, WasmId};
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
                Some(block_persisted) => match block_persisted {
                    MsgToProxy::BlockEvent(event) => {
                        for ue_event in event.ue_events {
                            self.on_ue_event(ue_event).await;
                        }
                        for act_event in event.act_events {
                            self.on_act_event(event.timestamp, act_event).await;
                        }
                        self.on_block_height_event(event.height).await;
                        for wasm_id in event.upgrade_wasm_ids {
                            self.on_upgrade_wasm_event(event.height, wasm_id).await;
                        }
                    }
                    MsgToProxy::WasmBinary(wasm_hash, wasm_binary) => {
                        self.on_upload_wasm_event(wasm_hash, wasm_binary).await;
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
    async fn on_block_height_event(&mut self, height: BlockHeight) {
        let output = InputOutputObject {
            action: ACTION_NEW_BLOCK_HEIGHT.to_owned(),
            proto: "".to_owned(),
            model: "".to_owned(),
            data: height.to_be_bytes().to_vec(),
            ext: vec![],
        };

        self.publish_vin_2_worker(None, &output).await;
    }

    async fn on_ue_event(&mut self, event: UpdateEntityEvent) {
        let UpdateEntityEvent {
            proto,
            model,
            req_id,
            entity_ids,
        } = event;

        for entity_id in entity_ids {
            let payload = payload_json(&req_id, entity_id);

            let output = InputOutputObject {
                action: ACTION_UPDATE_INDEX.to_owned(),
                proto: proto.clone(),
                model: model.clone(),
                data: payload.to_string().as_bytes().to_vec(),
                ext: vec![],
            };

            self.publish_vin_2_worker(Some(&proto), &output).await;
        }
    }

    async fn on_act_event(&mut self, timestamp: Timestamp, event: ActEvent) {
        let ext = json!({
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

    async fn on_upload_wasm_event(&mut self, wasm_hash: WasmHash, wasm_binary: Vec<u8>) {
        log::info!(
            "upload wasm event to worker, hash: {}, size: {}B",
            wasm_hash,
            wasm_binary.len()
        );

        let output = InputOutputObject {
            action: ACTION_UPLOAD_WASM.to_string(),
            proto: "".to_owned(),
            model: "".to_owned(),
            data: wasm_hash.as_bytes().into(),
            ext: wasm_binary,
        };
        self.publish_vin_2_worker(None, &output).await;
    }

    async fn on_upgrade_wasm_event(&mut self, block_height: BlockHeight, wasm_id: WasmId) {
        log::info!(
            "upgrade wasm event to worker, height: {}, proto: {}, hash: {}",
            block_height,
            wasm_id.proto,
            wasm_id.wasm_hash
        );

        let output = InputOutputObject {
            action: ACTION_UPGRADE_WASM.to_string(),
            proto: wasm_id.proto,
            model: "".to_owned(),
            data: wasm_id.wasm_hash.as_bytes().into(),
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
        if let Err(_err) = result {
            // log::error!("Error publishing to redis: {:?}", err);
        }
    }
}
