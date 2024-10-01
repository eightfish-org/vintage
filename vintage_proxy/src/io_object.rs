use futures::Stream;
use futures::StreamExt;
use redis::Msg;
use serde::{Deserialize, Serialize};
use serde_json::json;
use vintage_msg::{Action, EntityHash, EntityId, Model, Proto, ReqId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputOutputObject {
    pub action: Action,
    pub proto: Proto,
    pub model: Model,
    pub data: Vec<u8>,
    pub ext: Vec<u8>,
}

pub(crate) async fn read_msg(
    pubsub_stream: &mut (impl Stream<Item = Msg> + Unpin),
    channel_name: &str,
) -> anyhow::Result<InputOutputObject> {
    let msg = pubsub_stream.next().await;
    log::info!("received msg from channel {}", channel_name);

    let msg_payload: Vec<u8> = msg.unwrap().get_payload()?;
    let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();
    log::info!(
        "from redis, msg_obj: {} {} {} {} {}",
        msg_obj.action,
        msg_obj.proto,
        msg_obj.model,
        msg_obj.data.len(),
        msg_obj.ext.len()
    );
    Ok(msg_obj)
}

pub(crate) fn payload_json<TReqData>(req_id: &ReqId, req_data: TReqData) -> serde_json::Value
where
    TReqData: Serialize,
{
    json!({
        "reqid": req_id,
        "reqdata": req_data,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Payload<TReqData> {
    pub reqid: ReqId,
    pub reqdata: TReqData,
}

pub(crate) type EntitiesPayload = Payload<Vec<(EntityId, EntityHash)>>;
