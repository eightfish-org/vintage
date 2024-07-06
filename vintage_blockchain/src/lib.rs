mod block;
mod db;
mod genesis;
mod tx;

use crate::block::{block_handler, block_production_handler, BlockPool};
use crate::tx::{raw_tx_handler, tx_handler, TxPool};
use redb::Database;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{BlockChainMsg, BlockChainMsgChannels, NetworkMsg, WorkerMsg};

const DB_PATH: &str = "vintage.db";
const TX_POOL_CAPACITY: usize = 1000;
const BLOCK_POOL_CAPACITY: usize = 100;

#[allow(dead_code)]
pub struct BlockChain {
    database: Database,
    tx_pool: TxPool,
    block_pool: BlockPool,
    msg_receiver: mpsc::Receiver<BlockChainMsg>,
    worker_msg_sender: mpsc::Sender<WorkerMsg>,
    network_msg_sender: mpsc::Sender<NetworkMsg>,
}

impl BlockChain {
    pub fn create(channels: BlockChainMsgChannels) -> anyhow::Result<Self> {
        let database = Database::create(DB_PATH)?;
        Ok(Self {
            database,
            tx_pool: TxPool::new(TX_POOL_CAPACITY),
            block_pool: BlockPool::new(BLOCK_POOL_CAPACITY),
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
                    BlockChainMsg::RawTx(tx) => {
                        raw_tx_handler(
                            &self.database,
                            &mut self.tx_pool,
                            &self.network_msg_sender,
                            tx,
                        );
                    }
                    BlockChainMsg::Tx(tx) => {
                        tx_handler(&self.database, &mut self.tx_pool, tx);
                    }
                    BlockChainMsg::Block(block) => {
                        block_handler(&self.database, &mut self.block_pool, block);
                    }
                    BlockChainMsg::BlockProduction(block_production) => {
                        block_production_handler(block_production);
                    }
                },
                None => {
                    break;
                }
            }
        }
    }
}
