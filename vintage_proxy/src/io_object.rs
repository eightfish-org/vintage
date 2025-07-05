use anyhow::anyhow;
use futures::Stream;
use futures::StreamExt;
use redis::Msg;
use serde::{Deserialize, Serialize};
use vintage_msg::{Action, Model, Proto};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    if let Some(msg) = msg {
        log::info!("received msg from channel {}", channel_name);
        let msg_payload: Vec<u8> = msg.get_payload().expect("redis msg get_payload error.");
        let msg_obj: InputOutputObject =
            match serde_json::from_slice::<InputOutputObject>(&msg_payload) {
                Ok(msg_obj) => {
                    log::info!(
                        "from redis, msg_obj: {} {} {} {} {}",
                        msg_obj.action,
                        msg_obj.proto,
                        msg_obj.model,
                        msg_obj.data.len(),
                        msg_obj.ext.len()
                    );
                    msg_obj
                }
                Err(_) => {
                    log::error!("error when extract IO Object.");
                    InputOutputObject::default()
                }
            };
        Ok(msg_obj)
    } else {
        // if msg is None, it means the sub connection has been broken
        log::error!("received msg from channel {} is None.", channel_name);
        Err(anyhow!("msg from channel {} is None.", channel_name))
    }
}
