mod db;

use crate::db::AsyncStateDb;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_blockchain::BlockChainApi;
use vintage_msg::{ActEntitiesState, StateMsg, StateMsgChannels};

const STATE_DB_PATH: &str = "state.db";

pub struct State {
    msg_receiver: mpsc::Receiver<StateMsg>,
    blockchain_api: BlockChainApi,
    db: AsyncStateDb,
}

impl State {
    pub async fn create(
        channels: StateMsgChannels,
        blockchain_api: BlockChainApi,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            msg_receiver: channels.msg_receiver,
            blockchain_api,
            db: AsyncStateDb::create(STATE_DB_PATH).await?,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        tokio::spawn(self.service())
    }

    async fn service(mut self) {
        loop {
            match self.msg_receiver.recv().await {
                Some(msg) => match msg {
                    StateMsg::ActEntitiesState(state) => {
                        if let Err(err) = self.act_entities_state_handler(state).await {
                            log::error!("Failed to update entities state: {}", err);
                        }
                    }
                },
                None => {
                    break;
                }
            }
        }
    }

    async fn act_entities_state_handler(&mut self, state: ActEntitiesState) -> anyhow::Result<()> {
        self.db
            .insert_act_state(state.act_id, state.entities_state)
            .await?;

        let next_block_height = self.db.get_last_block_height().await? + 1;
        let act_ids = self
            .blockchain_api
            .get_block_act_ids(next_block_height)
            .await?;
        let acts = self.db.get_acts_state(act_ids).await?;

        self.db.write_state(next_block_height, acts).await
    }
}
