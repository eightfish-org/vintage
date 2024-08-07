mod act;
mod block;
mod db;

use crate::act::{act_handler, raw_act_handler, ActPool};
use crate::block::{block_msg_handler, BlockMsg, BlockMsgPool};
use crate::db::AsyncBlockChainDb;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{
    ActId, BlockChainMsg, BlockChainMsgChannels, BlockHeight, NetworkMsg, WorkerMsg,
};

const BLOCKCHAIN_DB_PATH: &str = "blockchain.db";
const BLOCK_POOL_CAPACITY: usize = 100;
const ACT_POOL_CAPACITY: usize = 2000;
const MAX_ACT_COUNT_PER_BLOCK: usize = 8000;

pub struct BlockChain {
    msg_receiver: mpsc::Receiver<BlockChainMsg>,
    #[allow(dead_code)]
    worker_msg_sender: mpsc::Sender<WorkerMsg>,
    network_msg_sender: mpsc::Sender<NetworkMsg>,
    db: AsyncBlockChainDb,
    act_pool: ActPool,
    block_msg_pool: BlockMsgPool,
}

impl BlockChain {
    pub async fn create(
        channels: BlockChainMsgChannels,
        db_path: String,
    ) -> anyhow::Result<(Self, BlockChainApi)> {
        let db_path = if db_path.is_empty() {
            BLOCKCHAIN_DB_PATH.to_string()
        } else {
            db_path
        };
        let db = AsyncBlockChainDb::create(db_path).await?;
        Ok((
            Self {
                msg_receiver: channels.msg_receiver,
                worker_msg_sender: channels.worker_msg_sender,
                network_msg_sender: channels.network_msg_sender,
                db: db.clone(),
                act_pool: ActPool::new(ACT_POOL_CAPACITY),
                block_msg_pool: BlockMsgPool::new(BLOCK_POOL_CAPACITY),
            },
            BlockChainApi::new(db),
        ))
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

#[derive(Clone)]
pub struct BlockChainApi {
    db: AsyncBlockChainDb,
}

impl BlockChainApi {
    fn new(db: AsyncBlockChainDb) -> Self {
        Self { db }
    }

    pub async fn get_block_act_ids(&self, block_height: BlockHeight) -> anyhow::Result<Vec<ActId>> {
        self.db.get_block_act_ids(block_height).await
    }
}
