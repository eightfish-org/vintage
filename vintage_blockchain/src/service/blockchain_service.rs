use crate::db::BlockChainDb;
use crate::network::{
    BroadcastMsg, MsgToNetworkSender, ReqBlock, ReqBlockHash, RequestMsg, RspBlock, RspBlockHash,
};
use crate::tx::{TxId, TxPool};
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_msg::{
    Act, Block, CalcHash, Hashed, MsgToBlockChain, NetworkRequestId, NodeId, UpdateEntityTx,
};
use vintage_utils::{BincodeDeserialize, Service};

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
                    MsgToBlockChain::Broadcast(msg_encoded) => {
                        if let Err(err) = self.broadcast_handler(msg_encoded).await {
                            log::error!("Failed to handle Broadcast: {:?}", err);
                        }
                    }
                    MsgToBlockChain::Request(node_id, request_id, request_encoded) => {
                        if let Err(err) = self
                            .request_handler(node_id, request_id, request_encoded)
                            .await
                        {
                            log::error!("Failed to handle Request: {:?}", err);
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
    async fn broadcast_handler(&self, msg_encoded: Vec<u8>) -> anyhow::Result<()> {
        let (msg, _bytes_read) = BroadcastMsg::bincode_deserialize(&msg_encoded)?;
        match msg {
            BroadcastMsg::Act(act) => {
                let act_id = self.put_act_to_pool(act).await?;
                log::info!("act from network: {}", act_id);
                Ok(())
            }
        }
    }

    async fn request_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        request_encoded: Vec<u8>,
    ) -> anyhow::Result<()> {
        let (msg, _bytes_read) = RequestMsg::bincode_deserialize(&request_encoded)?;
        match msg {
            RequestMsg::ReqBlockHash(req) => {
                self.request_block_hash_handler(node_id, request_id, req)
                    .await
            }
            RequestMsg::ReqBlock(req) => self.request_block_handler(node_id, request_id, req).await,
        }
    }

    async fn request_block_hash_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        req: ReqBlockHash,
    ) -> anyhow::Result<()> {
        log::info!("request_block_hash_handler from node: {}", node_id);
        let mut hash_list: Vec<Hashed> = Vec::new();
        for index in 0..req.count {
            match self.db.get_block(req.begin_height + index).await {
                Ok(block) => {
                    hash_list.push(block.hash);
                },
                Err(e) => {
                    log::warn!("Failed to get block at height {}: {:?}, skip", req.begin_height + index, e);
                    continue;
                }
            }
        }
        let rsp = RspBlockHash { hash_list };
        self.network_msg_sender
            .send_response(node_id, request_id, rsp);
        Ok(())
    }

    async fn request_block_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        req: ReqBlock,
    ) -> anyhow::Result<()> {
        log::info!("request_block_handler from node: {}", node_id);
        let mut block_list: Vec<Block> = Vec::new();
        for index in 0..req.count {
            let block = self.db.get_network_block(req.begin_height + index).await?;
            block_list.push(block);
        }
        let rsp = RspBlock { block_list };
        self.network_msg_sender
            .send_response(node_id, request_id, rsp);
        Ok(())
    }
}

// proxy
impl BlockChainService {
    async fn ue_tx_handler(&self, tx: UpdateEntityTx) -> anyhow::Result<()> {
        let tx_id = tx.calc_hash();
        self.db.insert_ue_tx_to_pool(tx_id, tx).await
    }

    async fn act_handler(&self, act: Act) -> anyhow::Result<()> {
        let act_id = self.put_act_to_pool(act.clone()).await?;
        log::info!("act from proxy: {}", act_id);
        self.network_msg_sender
            .send_broadcast(&BroadcastMsg::Act(act));
        Ok(())
    }
}

impl BlockChainService {
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
