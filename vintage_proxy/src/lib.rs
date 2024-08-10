mod data;
mod inbound;
mod outbound;

use crate::data::InputOutputObject;
use crate::inbound::{
    check_new_version_wasmfile, check_pair_list, disable_wasm_upgrade_flag, post,
    retreive_wasmfile, update_index,
};
use crate::outbound::{on_act_event, on_update_state_event};
use futures::StreamExt;
use redis::aio::{Connection, PubSub};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{BlockChainMsg, ProxyMsg, ProxyMsgChannels};

const SUBNODE_RPC_ENV: &str = "SUBNODE_RPC";
const REDIS_ADDRESS_ENV: &str = "REDIS_URL";

pub struct Proxy {
    msg_receiver: mpsc::Receiver<ProxyMsg>,
    blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
    redis_conn: Connection,
    pubsub_conn: PubSub,
    redis_conn_2: Connection,
}

impl Proxy {
    #[allow(unused_variables)]
    pub async fn create(channels: ProxyMsgChannels) -> anyhow::Result<Self> {
        let rpc_addr = std::env::var(SUBNODE_RPC_ENV)?;
        println!("rpc_addr: {}", rpc_addr);
        let rpc_addr2 = rpc_addr.clone();
        let redis_addr = std::env::var(REDIS_ADDRESS_ENV)?;
        println!("redis_addr: {}", redis_addr);
        let redis_client = redis::Client::open(redis_addr).unwrap();
        let redis_conn = redis_client.get_async_connection().await?;
        let redis_conn_2 = redis_client.get_async_connection().await?;
        let pubsub_conn = redis_client.get_async_connection().await?.into_pubsub();

        Ok(Self {
            msg_receiver: channels.msg_receiver,
            blockchain_msg_sender: channels.blockchain_msg_sender,
            redis_conn,
            pubsub_conn,
            redis_conn_2,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        let service = tokio::spawn(Self::service(
            self.blockchain_msg_sender,
            self.pubsub_conn,
            self.redis_conn_2,
        ));
        let service_2 = tokio::spawn(Self::service_2(self.msg_receiver, self.redis_conn));
        tokio::spawn(async {
            let _ = service_2.await;
            let _ = service.await;
        })
    }

    async fn service(
        blockchain_msg_sender: mpsc::Sender<BlockChainMsg>,
        mut pubsub_conn: PubSub,
        #[allow(unused_variables)] redis_conn: Connection,
    ) -> anyhow::Result<()> {
        pubsub_conn.subscribe("spin2proxy").await?;
        let mut pubsub_stream = pubsub_conn.on_message();

        loop {
            let msg = pubsub_stream.next().await;
            println!("received msg from channel spin2proxy: {:?}", msg);

            let msg_payload: Vec<u8> = msg.unwrap().get_payload()?;
            let msg_obj: InputOutputObject = serde_json::from_slice(&msg_payload).unwrap();
            println!("from redis: {:?}", msg_obj);

            if &msg_obj.action == "post" {
                post(&blockchain_msg_sender, msg_obj);
            } else if &msg_obj.action == "update_index" {
                update_index(&blockchain_msg_sender, msg_obj);
            } else if &msg_obj.action == "check_pair_list" {
                check_pair_list(msg_obj);
            } else if &msg_obj.action == "check_new_version_wasmfile" {
                check_new_version_wasmfile(msg_obj);
            } else if &msg_obj.action == "retreive_wasmfile" {
                retreive_wasmfile(msg_obj);
            } else if &msg_obj.action == "disable_wasm_upgrade_flag" {
                disable_wasm_upgrade_flag(msg_obj);
            }
        }
    }

    async fn service_2(mut msg_receiver: mpsc::Receiver<ProxyMsg>, mut redis_conn: Connection) {
        loop {
            match msg_receiver.recv().await {
                Some(block_persisted) => match block_persisted {
                    ProxyMsg::BlockEvent(event) => {
                        for update_state_event in event.entities {
                            on_update_state_event(&mut redis_conn, update_state_event).await
                        }
                        for act_event in event.acts {
                            on_act_event(&mut redis_conn, event.timestamp, act_event).await
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
