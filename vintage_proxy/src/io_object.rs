use futures::Stream;
use futures::StreamExt;
use redis::Msg;
use serde::{Deserialize, Serialize};
use vintage_msg::{Action, Model, Proto};

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
