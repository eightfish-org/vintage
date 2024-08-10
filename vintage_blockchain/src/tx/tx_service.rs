use crate::db::BlockChainDb;
use crate::tx::{calc_act_id, ActId, TxPool};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_msg::{Act, BlockChainMsg, NetworkMsg, UpdateEntities};
use vintage_utils::{SendMsg, Service};

pub struct TxService {
    db: BlockChainDb,
    tx_pool: Arc<TxPool>,
    msg_receiver: mpsc::Receiver<BlockChainMsg>,
    network_msg_sender: mpsc::Sender<NetworkMsg>,
}

impl TxService {
    pub(crate) fn new(
        db: BlockChainDb,
        tx_pool: Arc<TxPool>,

        msg_receiver: mpsc::Receiver<BlockChainMsg>,
        network_msg_sender: mpsc::Sender<NetworkMsg>,
    ) -> Self {
        Self {
            db,
            tx_pool,
            msg_receiver,
            network_msg_sender,
        }
    }
}

#[async_trait]
impl Service for TxService {
    type Output = ();

    async fn service(mut self) -> Self::Output {
        loop {
            match self.msg_receiver.recv().await {
                Some(msg) => match msg {
                    BlockChainMsg::ActFromNetwork(act) => {
                        if let Err(err) = self.network_act_handler(act).await {
                            log::error!("Failed to handle act ActFromNetwork: {:?}", err);
                        }
                    }
                    BlockChainMsg::Act(act) => {
                        if let Err(err) = self.act_handler(act).await {
                            log::error!("Failed to handle Act: {:?}", err);
                        }
                    }
                    BlockChainMsg::UpdateEntities(update_entities) => {
                        if let Err(err) = self.update_entities_handler(update_entities).await {
                            log::error!("Failed to handle UpdateEntities: {:?}", err);
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

// act
impl TxService {
    async fn network_act_handler(&self, act: Act) -> anyhow::Result<()> {
        let act_id = self.put_act_to_pool(act).await?;
        log::info!("act from network: {}", act_id);
        Ok(())
    }

    async fn act_handler(&self, act: Act) -> anyhow::Result<()> {
        let act_id = self.put_act_to_pool(act.clone()).await?;
        self.network_msg_sender
            .send_msg(NetworkMsg::BroadcastAct(act));
        log::info!("act from worker: {}", act_id);
        Ok(())
    }

    async fn put_act_to_pool(&self, act: Act) -> anyhow::Result<ActId> {
        let act_id = calc_act_id(&act);
        {
            if self.tx_pool.guard().acts.contains_act(&act_id) {
                return Err(anyhow!("act already exists in pool"));
            }
        }
        self.db.check_act_not_exists(act_id.clone()).await?;
        {
            self.tx_pool.guard().acts.insert_act(act_id.clone(), act);
        }
        Ok(act_id)
    }
}

// entity
impl TxService {
    async fn update_entities_handler(&self, update_entities: UpdateEntities) -> anyhow::Result<()> {
        Ok(())
    }
}
