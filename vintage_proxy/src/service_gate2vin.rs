use crate::constants::{ACTION_CHECK_PAIR_LIST, ACTION_POST, ACTION_UPDATE_INDEX};
use crate::io_object::read_msg;
use crate::{payload_json, EntitiesPayload, InputOutputObject};
use crate::{GATE_2_VIN, VIN_2_WORKER};
use async_trait::async_trait;
use redis::aio::{Connection, PubSub};
use redis::AsyncCommands;
use tokio::sync::mpsc;
use vintage_msg::{ActTx, BlockChainApi, Entity, MsgToBlockChain, UpdateEntityTx};
use vintage_utils::{SendMsg, Service};

pub struct Gate2Vin<TApi> {
    redis_conn: Connection,
    blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
    blockchain_api: TApi,
}

impl<TApi> Gate2Vin<TApi> {
    pub(crate) fn new(
        redis_conn: Connection,
        blockchain_msg_sender: mpsc::Sender<MsgToBlockChain>,
        blockchain_api: TApi,
    ) -> Self {
        Self {
            redis_conn,
            blockchain_msg_sender,
            blockchain_api,
        }
    }
}

#[async_trait]
impl<TApi> Service for Gate2Vin<TApi>
where
    TApi: BlockChainApi + Send + Sync,
{
    type Input = PubSub;
    type Output = anyhow::Result<()>;

    async fn service(mut self, mut pubsub: Self::Input) -> Self::Output {
        pubsub.subscribe(GATE_2_VIN).await?;
        let mut pubsub_stream = pubsub.on_message();

        loop {
            let msg_obj = read_msg(&mut pubsub_stream, GATE_2_VIN).await?;

            if &msg_obj.action == ACTION_POST {
                self.post(msg_obj);
            } else if &msg_obj.action == ACTION_UPDATE_INDEX {
                self.update_index(msg_obj);
            } else if &msg_obj.action == ACTION_CHECK_PAIR_LIST {
                if let Err(err) = self.check_pair_list(msg_obj).await {
                    log::error!("{} err: {:?}", ACTION_CHECK_PAIR_LIST, err)
                }
            }
        }
    }
}

impl<TApi> Gate2Vin<TApi>
where
    TApi: BlockChainApi,
{
    fn post(&self, object: InputOutputObject) {
        self.blockchain_msg_sender
            .send_msg(MsgToBlockChain::ActTx(ActTx {
                action: object.action,
                proto: object.proto,
                model: object.model,
                data: object.data,
            }));
    }

    fn update_index(&self, object: InputOutputObject) {
        let payload: EntitiesPayload = serde_json::from_slice(&object.data).unwrap();
        let entities = payload
            .reqdata
            .into_iter()
            .map(|(id, hash)| Entity { id, hash })
            .collect();

        self.blockchain_msg_sender
            .send_msg(MsgToBlockChain::UpdateEntityTx(UpdateEntityTx {
                proto: object.proto,
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
            .check_entities(msg_obj.proto.clone(), msg_obj.model.clone(), entities)
            .await;

        let ret_payload = payload_json(&payload.reqid, check_boolean.to_string());
        println!(
            "from redis: check_pair_list: ret_payload: {:?}",
            ret_payload
        );

        // send packet back to the spin runtime
        let output = InputOutputObject {
            action: msg_obj.action,
            proto: msg_obj.proto.clone(),
            model: msg_obj.model,
            data: ret_payload.to_string().as_bytes().to_vec(),
            ext: vec![],
        };
        let output_string = serde_json::to_vec(&output).unwrap();
        let channel = format!("{VIN_2_WORKER}:{}", msg_obj.proto);
        let result: Result<u32, redis::RedisError> =
            self.redis_conn.publish(&channel, output_string).await;
        result?;
        Ok(())
    }
}
