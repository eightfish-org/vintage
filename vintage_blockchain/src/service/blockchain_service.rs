use crate::db::BlockChainDb;
use crate::network::{BroadcastMsg, MsgToNetworkSender, ReqBlock, ReqBlockHash, RequestMsg};
use crate::proxy::MsgToProxySender;
use crate::tx::{TxId, TxPool};
use crate::wasm_db::WasmDb;
use anyhow::anyhow;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_msg::{
    ActTx, Block, BlockHash, MsgToBlockChain, NetworkRequestId, NodeId, UpdateEntityTx, UploadWasm,
    WasmHash, WasmId, WasmInfo,
};
use vintage_utils::{BincodeDeserialize, CalcHash, Service};

pub struct BlockChainService {
    blockchain_db: BlockChainDb,
    wasm_db: WasmDb,
    tx_pool: Arc<TxPool>,
    msg_receiver: mpsc::Receiver<MsgToBlockChain>,
    proxy_msg_sender: MsgToProxySender,
    network_msg_sender: MsgToNetworkSender,
}

impl BlockChainService {
    pub(crate) fn new(
        blockchain_db: BlockChainDb,
        wasm_db: WasmDb,
        tx_pool: Arc<TxPool>,
        msg_receiver: mpsc::Receiver<MsgToBlockChain>,
        proxy_msg_sender: MsgToProxySender,
        network_msg_sender: MsgToNetworkSender,
    ) -> Self {
        Self {
            blockchain_db,
            wasm_db,
            tx_pool,
            msg_receiver,
            proxy_msg_sender,
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
                    MsgToBlockChain::Broadcast(node_id, msg_encoded) => {
                        if let Err(err) = self.broadcast_handler(node_id, msg_encoded).await {
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
                    MsgToBlockChain::ActTx(act_tx) => {
                        if let Err(err) = self.act_handler(act_tx).await {
                            log::error!("Failed to handle ActTx: {:?}", err);
                        }
                    }
                    MsgToBlockChain::UpdateEntityTx(tx) => {
                        if let Err(err) = self.ue_tx_handler(tx).await {
                            log::error!("Failed to handle UpdateEntityTx: {:?}", err);
                        }
                    }
                    MsgToBlockChain::UploadWasm(upload_wasm) => {
                        if let Err(err) = self.upload_wasm_handler(upload_wasm).await {
                            log::error!("Failed to handle UploadWasm: {:?}", err);
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
    async fn broadcast_handler(
        &self,
        _node_id: NodeId,
        msg_encoded: Vec<u8>,
    ) -> anyhow::Result<()> {
        let (msg, _bytes_read) = BroadcastMsg::bincode_deserialize(&msg_encoded)?;
        match msg {
            BroadcastMsg::ActTx(act_tx) => {
                let act_tx_id = self.put_act_tx_to_pool(act_tx).await?;
                log::info!("act tx from network: {}", act_tx_id);
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
            RequestMsg::ReqWasmExists(wasm_hash) => {
                self.request_wasm_exists_handler(node_id, request_id, wasm_hash)
                    .await
            }
            RequestMsg::ReqWasm(wasm_hash) => {
                self.request_wasm_handler(node_id, request_id, wasm_hash)
                    .await
            }
        }
    }

    async fn request_block_hash_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        req: ReqBlockHash,
    ) -> anyhow::Result<()> {
        log::debug!("request_block_hash_handler from node: {}", node_id);
        let mut hash_list: Vec<BlockHash> = Vec::new();
        for index in 0..req.count {
            match self.blockchain_db.get_block(req.begin_height + index).await {
                Ok(block) => {
                    hash_list.push(block.hash);
                }
                Err(e) => {
                    log::info!(
                        "Failed to get block at height {}: error:{:?}, break",
                        req.begin_height + index,
                        e
                    );
                    break;
                }
            }
        }
        self.network_msg_sender
            .send_response(node_id, request_id, hash_list);
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
            let block = self
                .blockchain_db
                .get_network_block(req.begin_height + index)
                .await?;
            block_list.push(block);
        }
        self.network_msg_sender
            .send_response(node_id, request_id, block_list);
        Ok(())
    }

    async fn request_wasm_exists_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        wasm_hash: WasmHash,
    ) -> anyhow::Result<()> {
        let exists = self.wasm_db.wasm_binary_exists(wasm_hash).await?;
        self.network_msg_sender
            .send_response(node_id, request_id, exists);
        Ok(())
    }

    async fn request_wasm_handler(
        &self,
        node_id: NodeId,
        request_id: NetworkRequestId,
        wasm_hash: WasmHash,
    ) -> anyhow::Result<()> {
        let wasm_binary = self.wasm_db.get_wasm_binary(wasm_hash).await?;
        self.network_msg_sender
            .send_response(node_id, request_id, wasm_binary);
        Ok(())
    }
}

// worker
impl BlockChainService {
    async fn ue_tx_handler(&self, tx: UpdateEntityTx) -> anyhow::Result<()> {
        let tx_id = tx.calc_hash();
        self.blockchain_db.insert_ue_tx_to_pool(tx_id, tx).await
    }

    async fn act_handler(&self, act_tx: ActTx) -> anyhow::Result<()> {
        let act_tx_id = self.put_act_tx_to_pool(act_tx.clone()).await?;
        log::info!("act tx from proxy: {}", act_tx_id);
        self.network_msg_sender
            .send_broadcast(&BroadcastMsg::ActTx(act_tx));
        Ok(())
    }

    async fn put_act_tx_to_pool(&self, act_tx: ActTx) -> anyhow::Result<TxId> {
        let act_tx_id = act_tx.calc_hash();
        {
            if self.tx_pool.act_txs_guard().contains_key(&act_tx_id) {
                return Err(anyhow!("act tx already exists in pool"));
            }
        }
        self.blockchain_db
            .check_act_not_exists(act_tx_id.clone())
            .await?;
        {
            self.tx_pool
                .act_txs_guard()
                .insert(act_tx_id.clone(), act_tx);
        }
        Ok(act_tx_id)
    }
}

// admin
impl BlockChainService {
    async fn upload_wasm_handler(&self, upload_wasm: UploadWasm) -> anyhow::Result<()> {
        let UploadWasm {
            proto,
            wasm_binary,
            block_interval,
        } = upload_wasm;

        let wasm_hash = wasm_binary.calc_hash();
        log::info!("wasm tx, proto: {}, hash: {}", proto, wasm_hash);

        if self
            .wasm_db
            .try_insert_wasm_binary(wasm_hash.clone(), wasm_binary.clone())
            .await?
        {
            log::info!("wasm file {} saved", wasm_hash);
            self.proxy_msg_sender
                .send_wasm_binary(wasm_hash.clone(), wasm_binary);
        } else {
            log::info!("wasm file {} already exists", wasm_hash);
        }

        self.put_wasm_tx_to_pool(WasmId { proto, wasm_hash }, WasmInfo { block_interval })
            .await?;

        Ok(())
    }

    async fn put_wasm_tx_to_pool(&self, key: WasmId, info: WasmInfo) -> anyhow::Result<()> {
        {
            if self.tx_pool.wasm_txs_guard().contains_key(&key) {
                return Err(anyhow!("wasm tx already exists in pool"));
            }
        }
        self.blockchain_db
            .check_wasm_tx_not_exists(key.clone())
            .await?;
        {
            self.tx_pool.wasm_txs_guard().insert(key, info);
        }
        Ok(())
    }
}
