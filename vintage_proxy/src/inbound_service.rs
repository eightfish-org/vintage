use crate::data::{payload_json, EntitiesPayload, InputOutputObject};
use async_trait::async_trait;
use futures::StreamExt;
use redis::aio::{Connection, PubSub};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_msg::{Act, BlockChainApi, BlockChainMsg, Entity, UpdateEntityTx};
use vintage_utils::{SendMsg, Service};

pub struct ProxyInboundService<TApi> {
    redis_conn: Connection,
    blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
    blockchain_api: Arc<TApi>,
}

impl<TApi> ProxyInboundService<TApi> {
    pub(crate) fn new(
        redis_conn: Connection,
        blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
        blockchain_api: Arc<TApi>,
    ) -> Self {
        Self {
            redis_conn,
            blockchain_msg_sender,
            blockchain_api,
        }
    }
}

#[async_trait]
impl<TApi> Service for ProxyInboundService<TApi>
where
    TApi: BlockChainApi + Send + Sync,
{
    type Input = PubSub;
    type Output = anyhow::Result<()>;

    async fn service(mut self, mut pubsub_conn: Self::Input) -> Self::Output {
        pubsub_conn.subscribe("spin2proxy").await?;
        let mut pubsub_stream = pubsub_conn.on_message();

        loop {
            let msg = pubsub_stream.next().await;
            println!("received msg from channel spin2proxy: {:?}", msg);

            let msg_payload: Vec<u8> = msg.unwrap().get_payload()?;
            let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();
            println!("from redis: {:?}", msg_obj);

            if &msg_obj.action == "post" {
                self.post(msg_obj);
            } else if &msg_obj.action == "update_index" {
                self.update_index(msg_obj);
            } else if &msg_obj.action == "check_pair_list" {
                if let Err(err) = self.check_pair_list(msg_obj).await {
                    log::error!("check_pair_list err: {:?}", err)
                }
            } else if &msg_obj.action == "check_new_version_wasmfile" {
                self.check_new_version_wasmfile(msg_obj);
            } else if &msg_obj.action == "retreive_wasmfile" {
                self.retreive_wasmfile(msg_obj);
            } else if &msg_obj.action == "disable_wasm_upgrade_flag" {
                self.disable_wasm_upgrade_flag(msg_obj);
            }
        }
    }
}

impl<TApi> ProxyInboundService<TApi>
where
    TApi: BlockChainApi,
{
    fn post(&self, object: InputOutputObject) {
        self.blockchain_msg_sender.send_msg(BlockChainMsg::Act(Act {
            kind: object.action,
            model: object.model,
            data: object.data,
        }))
    }

    fn update_index(&self, object: InputOutputObject) {
        let payload: EntitiesPayload = serde_json::from_slice(&object.data).unwrap();
        let entities = payload
            .reqdata
            .into_iter()
            .map(|(id, hash)| Entity { id, hash })
            .collect();

        self.blockchain_msg_sender
            .send_msg(BlockChainMsg::UpdateEntityTx(UpdateEntityTx {
                model: object.model,
                req_id: payload.reqid,
                entities,
            }));
    }

    async fn check_pair_list(&mut self, msg_obj: InputOutputObject) -> anyhow::Result<()> {
        let payload: EntitiesPayload = serde_json::from_slice(&msg_obj.data).unwrap();
        let entities = payload
            .reqdata
            .into_iter()
            .map(|(id, hash)| Entity { id, hash })
            .collect();

        let check_boolean: bool = self
            .blockchain_api
            .check_entities(msg_obj.model.clone(), entities)
            .await;

        let ret_payload = payload_json(&payload.reqid, check_boolean.to_string());
        println!(
            "from redis: check_pair_list: ret_payload: {:?}",
            ret_payload
        );

        // send packet back to the spin runtime
        let output = InputOutputObject {
            action: msg_obj.action,
            model: msg_obj.model,
            data: ret_payload.to_string().as_bytes().to_vec(),
            ext: vec![],
        };
        let output_string = serde_json::to_vec(&output).unwrap();
        self.redis_conn.publish("proxy2spin", output_string).await?;
        Ok(())
    }

    fn check_new_version_wasmfile(&self, _object: InputOutputObject) {}

    fn retreive_wasmfile(&self, _object: InputOutputObject) {}

    fn disable_wasm_upgrade_flag(&self, _object: InputOutputObject) {}
}
