mod block;
mod db;
mod act;

use crate::block::{block_msg_handler, BlockMsg, BlockMsgPool};
use crate::db::AsyncBlockChainDb;
use crate::act::{raw_act_handler, act_handler, ActPool};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{BlockChainMsg, BlockChainMsgChannels, NetworkMsg, WorkerMsg};

const BLOCKCHAIN_DB_PATH: &str = "blockchain.db";
const BLOCK_POOL_CAPACITY: usize = 100;
const ACT_POOL_CAPACITY: usize = 2000;
const MAX_ACT_COUNT_PER_BLOCK: usize = 8000;

pub struct BlockChain {
    db: AsyncBlockChainDb,
    act_pool: ActPool,
    block_msg_pool: BlockMsgPool,
    msg_receiver: mpsc::Receiver<BlockChainMsg>,
    #[allow(dead_code)]
    worker_msg_sender: mpsc::Sender<WorkerMsg>,
    network_msg_sender: mpsc::Sender<NetworkMsg>,
}

impl BlockChain {
    pub async fn create(channels: BlockChainMsgChannels, db_path: String) -> anyhow::Result<Self> {
        let db_path = if db_path.is_empty() { BLOCKCHAIN_DB_PATH.to_string() } else { db_path };
        Ok(Self {
            db: AsyncBlockChainDb::create(db_path).await?,
            act_pool: ActPool::new(ACT_POOL_CAPACITY),
            block_msg_pool: BlockMsgPool::new(BLOCK_POOL_CAPACITY),
            msg_receiver: channels.msg_receiver,
            worker_msg_sender: channels.worker_msg_sender,
            network_msg_sender: channels.network_msg_sender,
        })
    }

    pub fn start_service(self) -> JoinHandle<()> {
        tokio::spawn(self.service())
    }

    async fn service(mut self) {
        loop {
            match self.msg_receiver.recv().await {
                Some(msg) => match msg {
                    BlockChainMsg::RawAct(act) => {
                        if let Err(err) = raw_act_handler(
                            &self.db,
                            &mut self.act_pool,
                            &self.network_msg_sender,
                            act,
                        )
                        .await
                        {
                            log::error!("Failed to handle raw act: {}", err);
                        }
                    }
                    BlockChainMsg::Act(act) => {
                        if let Err(err) = act_handler(&self.db, &mut self.act_pool, act).await {
                            log::error!("Failed to handle act: {}", err);
                        }
                    }
                    BlockChainMsg::ImportBlock(block) => {
                        if let Err(err) = block_msg_handler(
                            &self.db,
                            &mut self.act_pool,
                            &mut self.block_msg_pool,
                            &self.network_msg_sender,
                            BlockMsg::ImportBlock(block),
                        )
                        .await
                        {
                            log::error!("Failed to handle block: {}", err);
                        }
                    }
                    BlockChainMsg::ProduceBlock(block_production) => {
                        if let Err(err) = block_msg_handler(
                            &self.db,
                            &mut self.act_pool,
                            &mut self.block_msg_pool,
                            &self.network_msg_sender,
                            BlockMsg::ProduceBlock(block_production),
                        )
                        .await
                        {
                            log::error!("Failed to handle block production: {}", err);
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
