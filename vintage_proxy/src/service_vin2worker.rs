use crate::constants::{ACTION_NEW_BLOCK_HEIGHT, ACTION_UPDATE_INDEX};
use crate::VIN_2_WORKER;
use crate::{payload_json, InputOutputObject};
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json::json;
use tokio::sync::mpsc;
use vintage_msg::{ActEvent, BlockHeight, MsgToProxy, Proto, UpdateEntityEvent};
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

        let proto = event.act.proto.clone();
        let output = InputOutputObject {
            action: event.act.action,
            proto: event.act.proto,
            model: event.act.model,
            data: event.act.data,
            ext: ext.to_string().as_bytes().to_vec(),
        };

        self.publish_vin_2_worker(Some(&proto), &output).await;
    }

    async fn publish_vin_2_worker(&mut self, proto: Option<&Proto>, output: &InputOutputObject) {
        let channel = match proto {
            Some(value) => format!("{}:{}", VIN_2_WORKER, value),
            None => VIN_2_WORKER.to_owned(),
        };
        let output_bytes = serde_json::to_vec(output).unwrap();

        let result: Result<String, redis::RedisError> =
            self.redis_conn.publish(channel, output_bytes).await;
        if let Err(err) = result {
            log::error!("Error publishing to redis: {:?}", err);
        }
    }
}
