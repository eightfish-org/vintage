use crate::db::BlockChainDb;
use crate::network::{msg_decode, MsgDecoded, MsgToNetworkSender};
use crate::tx::{TxId, TxPool};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_msg::{Act, CalcHash, MsgToBlockChain, NodeId, UpdateEntityTx};
use vintage_utils::Service;

pub struct BlockChainService {
    db: BlockChainDb,
    tx_pool: Arc<TxPool>,
    msg_receiver: mpsc::Receiver<MsgToBlockChain>,
    network_msg_sender: MsgToNetworkSender,
}

impl BlockChainService {
    pub(crate) fn new(
        db: BlockChainDb,
        tx_pool: Arc<TxPool>,
        msg_receiver: mpsc::Receiver<MsgToBlockChain>,
        network_msg_sender: MsgToNetworkSender,
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
impl Service for BlockChainService {
    type Input = ();
    type Output = ();

    async fn service(mut self, _input: Self::Input) -> Self::Output {
        loop {
            match self.msg_receiver.recv().await {
                Some(msg) => match msg {
                    MsgToBlockChain::NetworkMsg((node_id, msg_encoded)) => {
                        if let Err(err) = self.network_msg_handler(node_id, msg_encoded).await {
                            log::error!("Failed to handle NetworkMsg: {:?}", err);
                        }
                    }
                    MsgToBlockChain::Act(act) => {
                        if let Err(err) = self.act_handler(act).await {
                            log::error!("Failed to handle Act: {:?}", err);
                        }
                    }
                    MsgToBlockChain::UpdateEntityTx(tx) => {
                        if let Err(err) = self.ue_tx_handler(tx).await {
                            log::error!("Failed to handle UpdateEntityTx: {:?}", err);
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

// network
impl BlockChainService {
    async fn network_msg_handler(
        &self,
        _node_id: NodeId,
        msg_encoded: Vec<u8>,
    ) -> anyhow::Result<()> {
        let msg_decoded = msg_decode(&msg_encoded)?;
        match msg_decoded {
            MsgDecoded::Act(act) => {
                let act_id = self.put_act_to_pool(act).await?;
                log::info!("act from network: {}", act_id);
            }
        }
        Ok(())
    }
}

// act
impl BlockChainService {
    async fn act_handler(&self, act: Act) -> anyhow::Result<()> {
        let act_id = self.put_act_to_pool(act.clone()).await?;
        log::info!("act from proxy: {}", act_id);
        self.network_msg_sender.broadcast_msg(&act);
        Ok(())
    }

    async fn put_act_to_pool(&self, act: Act) -> anyhow::Result<TxId> {
        let act_id = act.calc_hash();
        {
            if self.tx_pool.guard().contains_act(&act_id) {
                return Err(anyhow!("act already exists in pool"));
            }
        }
        self.db.check_act_not_exists(act_id.clone()).await?;
        {
            self.tx_pool.guard().insert_act(act_id.clone(), act);
        }
        Ok(act_id)
    }
}

// update entity
impl BlockChainService {
    async fn ue_tx_handler(&self, tx: UpdateEntityTx) -> anyhow::Result<()> {
        let tx_id = tx.calc_hash();
        self.db.insert_ue_tx_to_pool(tx_id, tx).await
    }
}
