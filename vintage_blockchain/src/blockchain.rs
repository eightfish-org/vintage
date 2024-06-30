use crate::db::DB_PATH;
use crate::{BlockValidate, TxValidate};
use redb::Database;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use vintage_msg::{BlockChainMsg, BlockChainMsgChannels, NetworkMsg, WorkerMsg};

#[allow(dead_code)]
pub struct BlockChain {
    database: Database,
}

impl BlockChain {
    pub fn create(channels: BlockChainMsgChannels) -> anyhow::Result<Self> {
        let database = Database::create(DB_PATH)?;
        Ok(Self::new(
            database,
            channels.msg_receiver,
            channels.worker_msg_sender,
            channels.network_msg_sender,
        ))
    }

    fn new(
        database: Database,
        msg_receiver: mpsc::Receiver<BlockChainMsg>,
        worker_msg_sender: mpsc::Sender<WorkerMsg>,
        network_msg_sender: mpsc::Sender<NetworkMsg>,
    ) -> Self {
        Self { database }
    }

    pub fn start_service(self) -> JoinHandle<()> {
        tokio::spawn(self.service())
    }

    #[allow(unused_variables)]
    async fn service(self) {
        let tx_validate = TxValidate::new(&self.database);
        let block_validate = BlockValidate::new(&self.database);
    }
}
