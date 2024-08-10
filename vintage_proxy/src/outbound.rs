use crate::data::InputOutputObject;
use redis::aio::Connection;
use redis::AsyncCommands;
use serde_json::json;
use vintage_msg::{ActEvent, EntityEvent};
use vintage_utils::Timestamp;

pub(crate) async fn on_act_event(
    redis_conn: &mut Connection,
    timestamp: Timestamp,
    event: ActEvent,
) {
    let ext = json!({
        "time": timestamp,
        "nonce": event.nonce,
        "randomvec": event.random,
    });

    let output = InputOutputObject {
        action: event.act.kind,
        model: event.act.model,
        data: event.act.data,
        ext: ext.to_string().as_bytes().to_vec(),
    };

    let output_vec = serde_json::to_vec(&output).unwrap();
    let _: Result<String, redis::RedisError> = redis_conn.publish("proxy2spin", output_vec).await;
}

pub(crate) async fn on_update_state_event(redis_conn: &mut Connection, event: EntityEvent) {
    let payload = json!({
        "reqid": event.req_id,
        "reqdata": event.entity_id,
    });

    let output = InputOutputObject {
        model: event.model,
        action: "update_index".to_owned(),
        data: payload.to_string().as_bytes().to_vec(),
        ext: vec![],
    };

    let output_vec = serde_json::to_vec(&output).unwrap();
    let _: Result<String, redis::RedisError> = redis_conn.publish("proxy2spin", output_vec).await;
}
