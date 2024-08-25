use crate::{payload_json, InputOutputObject};
use async_trait::async_trait;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json::json;
use tokio::sync::mpsc;
use vintage_msg::{ActEvent, MsgToProxy, UpdateEntityEvent};
use vintage_utils::{Service, Timestamp};

pub struct ProxyOutboundService {
    redis_conn: Connection,
    msg_receiver: mpsc::Receiver<MsgToProxy>,
}

impl ProxyOutboundService {
    pub(crate) fn new(redis_conn: Connection, msg_receiver: mpsc::Receiver<MsgToProxy>) -> Self {
        Self {
            redis_conn,
            msg_receiver,
        }
    }
}

#[async_trait]
impl Service for ProxyOutboundService {
    type Input = ();
    type Output = ();

    async fn service(mut self, _input: Self::Input) -> Self::Output {
        loop {
            match self.msg_receiver.recv().await {
                Some(block_persisted) => match block_persisted {
                    MsgToProxy::BlockEvent(event) => {
                        for ue_event in event.ue_events {
                            self.on_ue_event(ue_event).await
                        }
                        for act_event in event.act_events {
                            self.on_act_event(event.timestamp, act_event).await
                        }
                    }
                },
                None => {
                    break;
                }
            }
        }
    }
}

impl ProxyOutboundService {
    async fn on_act_event(&mut self, timestamp: Timestamp, event: ActEvent) {
        let ext = json!({
            "time": timestamp,
            "nonce": event.act_number,
            "randomvec": event.random,
        });

        let output = InputOutputObject {
            action: event.act.kind,
            model: event.act.model,
            data: event.act.data,
            ext: ext.to_string().as_bytes().to_vec(),
        };

        let output_vec = serde_json::to_vec(&output).unwrap();
        let _: Result<String, redis::RedisError> =
            self.redis_conn.publish("proxy2spin", output_vec).await;
    }

    async fn on_ue_event(&mut self, event: UpdateEntityEvent) {
        let UpdateEntityEvent {
            model,
            req_id,
            entity_ids,
        } = event;

        for entity_id in entity_ids {
            let payload = payload_json(&req_id, entity_id);

            let output = InputOutputObject {
                action: "update_index".to_owned(),
                model: model.clone(),
                data: payload.to_string().as_bytes().to_vec(),
                ext: vec![],
            };

            let output_vec = serde_json::to_vec(&output).unwrap();
            let _: Result<String, redis::RedisError> =
                self.redis_conn.publish("proxy2spin", output_vec).await;
        }
    }
}
