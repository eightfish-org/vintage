mod block;
mod db;
mod genesis;
mod tx;

use crate::block::{block_msg_handler, BlockMsg, BlockMsgPool};
use crate::db::Db;
use crate::tx::{raw_tx_handler, tx_handler, TxPool};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{BlockChainMsg, BlockChainMsgChannels, NetworkMsg, WorkerMsg};

const DB_PATH: &str = "vintage.db";
const TX_POOL_CAPACITY: usize = 1000;
const BLOCK_POOL_CAPACITY: usize = 100;
const MAX_TXS_PER_BLOCK: usize = 1000;

pub struct BlockChain {
    db: Db,
    tx_pool: TxPool,
    block_msg_pool: BlockMsgPool,
    msg_receiver: mpsc::Receiver<BlockChainMsg>,
    #[allow(dead_code)]
    worker_msg_sender: mpsc::Sender<WorkerMsg>,
    network_msg_sender: mpsc::Sender<NetworkMsg>,
}

impl BlockChain {
    pub fn create(channels: BlockChainMsgChannels) -> anyhow::Result<Self> {
        let db = Db::create(DB_PATH)?;
        Ok(Self {
            db,
            tx_pool: TxPool::new(TX_POOL_CAPACITY),
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
                    BlockChainMsg::RawTx(tx) => {
                        if let Err(err) = raw_tx_handler(
                            &self.db,
                            &mut self.tx_pool,
                            &self.network_msg_sender,
                            tx,
                        ) {
                            log::error!("Failed to handle raw tx: {}", err);
                        }
                    }
                    BlockChainMsg::Tx(tx) => {
                        if let Err(err) = tx_handler(&self.db, &mut self.tx_pool, tx) {
                            log::error!("Failed to handle tx: {}", err);
                        }
                    }
                    BlockChainMsg::Block(block) => {
                        if let Err(err) = block_msg_handler(
                            &self.db,
                            &mut self.tx_pool,
                            &mut self.block_msg_pool,
                            &self.network_msg_sender,
                            BlockMsg::Block(block),
                        ) {
                            log::error!("Failed to handle block: {}", err);
                        }
                    }
                    BlockChainMsg::BlockProduction(block_production) => {
                        if let Err(err) = block_msg_handler(
                            &self.db,
                            &mut self.tx_pool,
                            &mut self.block_msg_pool,
                            &self.network_msg_sender,
                            BlockMsg::BlockProduction(block_production),
                        ) {
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
