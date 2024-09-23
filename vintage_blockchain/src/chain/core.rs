use crate::BlockState;
use crate::DownloadWasmTask;
use crate::MsgToProxySender;
use crate::WasmDb;
use crate::{get_act_txs_from_pool, get_wasm_txs_from_pool, remove_txs_from_pool, TxId, TxPool};
use crate::{BlockChainDb, BlockInDb};
use crate::{MAX_ACT_COUNT_PER_BLOCK, MAX_UE_TX_COUNT_PER_BLOCK};
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use vintage_msg::{ActTx, Block, BlockHash, BlockHeight, WasmId, WasmTx};
use vintage_utils::{current_timestamp, CalcHash, Hashed, ServiceStarter, Timestamp};

pub type ArcBlockChainCore = Arc<tokio::sync::Mutex<BlockChainCore>>;

pub struct BlockChainCore {
    blockchain_db: BlockChainDb,
    wasm_db: WasmDb,
    tx_pool: Arc<TxPool>,
    proxy_msg_sender: MsgToProxySender,
    last_commited_time: Timestamp,
}

impl BlockChainCore {
    pub(crate) fn new(
        blockchain_db: BlockChainDb,
        wasm_db: WasmDb,
        tx_pool: Arc<TxPool>,
        proxy_msg_sender: MsgToProxySender,
    ) -> Self {
        Self {
            blockchain_db,
            wasm_db,
            tx_pool,
            proxy_msg_sender,
            last_commited_time: 0,
        }
    }
}

impl BlockChainCore {
    pub(crate) fn get_last_commited_time(&self) -> Timestamp {
        self.last_commited_time
    }

    pub(crate) async fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        self.blockchain_db.get_block_height().await
    }

    pub(crate) async fn new_block(&self, height: u64) -> anyhow::Result<(Block, Hashed)> {
        self.check_block_height(height).await?;
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_tx_ids, act_txs) =
            { get_act_txs_from_pool(&self.tx_pool.act_txs_guard(), MAX_ACT_COUNT_PER_BLOCK) };
        let (ue_tx_ids, ue_txs) = self
            .blockchain_db
            .get_ue_txs_in_pool(MAX_UE_TX_COUNT_PER_BLOCK)
            .await?;
        let wasm_txs = { get_wasm_txs_from_pool(&self.tx_pool.wasm_txs_guard()) };

        // state
        let state = Self::block_state(&prev_block, &act_txs);

        // hash
        let timestamp = current_timestamp();
        let hash = Self::calc_block_hash(
            height,
            timestamp,
            &state,
            &act_tx_ids,
            &ue_tx_ids,
            &wasm_txs,
            &prev_block.hash,
        );

        // new block
        Ok((
            Block {
                timestamp,
                act_txs,
                ue_txs,
                wasm_txs,
            },
            hash,
        ))
    }

    pub(crate) async fn check_block(
        &self,
        height: u64,
        block: &Block,
        hash: &Hashed,
    ) -> anyhow::Result<()> {
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_tx_ids, ue_tx_ids, wasm_ids) = Self::tx_keys_of(block);
        self.blockchain_db
            .check_act_txs_not_exist(act_tx_ids.clone())
            .await?;
        self.blockchain_db
            .check_ue_txs_not_exist(ue_tx_ids.clone())
            .await?;
        self.check_ue_txs_exist_in_pool(ue_tx_ids.clone()).await?;
        self.blockchain_db
            .check_wasm_txs_not_exist(wasm_ids)
            .await?;

        // state
        let state = Self::block_state(&prev_block, &block.act_txs);

        // hash
        let calc_hash = Self::calc_block_hash(
            height,
            block.timestamp,
            &state,
            &act_tx_ids,
            &ue_tx_ids,
            &block.wasm_txs,
            &prev_block.hash,
        );
        if *hash == calc_hash {
            Ok(())
        } else {
            Err(anyhow!("block hash, {} != {}", hash, calc_hash).into())
        }
    }

    pub(crate) async fn commit_block(
        &mut self,
        height: u64,
        block: Block,
        hash: Hashed,
    ) -> anyhow::Result<()> {
        self.check_block_height(height).await?;
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_tx_ids, ue_tx_ids, wasm_ids) = Self::tx_keys_of(&block);
        let act_txs = block.act_txs.clone();
        let ue_txs = block.ue_txs.clone();

        // state
        let state = Self::block_state(&prev_block, &block.act_txs);

        // commit block
        let block_hash_cloned = hash.clone();
        let timestamp = block.timestamp;
        let total_act_txs = state.total_act_txs;
        self.try_insert_download_wasm_tasks(&wasm_ids).await;
        self.blockchain_db
            .commit_block(
                height,
                hash,
                state,
                act_tx_ids.clone(),
                ue_tx_ids,
                wasm_ids.clone(),
                block,
            )
            .await?;
        log::info!(
            "block commited, height: {},\nhash: {}, timestamp: {}, total_act_txs: {}, act_txs: {}, ue_txs: {}",
            height,
            block_hash_cloned,
            timestamp,
            total_act_txs,
            act_txs.len(),
            ue_txs.len()
        );

        // after - commit block
        self.last_commited_time = current_timestamp();
        {
            remove_txs_from_pool(&mut self.tx_pool.act_txs_guard(), &act_tx_ids);
        }
        {
            remove_txs_from_pool(&mut self.tx_pool.wasm_txs_guard(), &wasm_ids);
        }
        let upgrade_wasm_ids = self.blockchain_db.get_upgrade_wasm_ids(height).await?;
        self.proxy_msg_sender.send_block_event(
            height,
            timestamp,
            total_act_txs,
            act_txs,
            ue_txs,
            upgrade_wasm_ids,
        );

        Ok(())
    }

    pub(crate) async fn import_block(
        &mut self,
        block_height: BlockHeight,
        block: Block,
        hash: BlockHash,
    ) -> anyhow::Result<()> {
        self.check_block(block_height, &block, &hash).await?;
        self.commit_block(block_height, block, hash).await?;
        Ok(())
    }
}

impl BlockChainCore {
    async fn check_block_height(&self, height: BlockHeight) -> anyhow::Result<()> {
        let last_height = self.blockchain_db.get_block_height().await?;
        if height == last_height + 1 {
            Ok(())
        } else {
            Err(anyhow!("height is {}, last height is {}", height, last_height).into())
        }
    }

    async fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        self.blockchain_db.get_block(height).await
    }

    async fn check_ue_txs_exist_in_pool(&self, ue_tx_ids: Vec<TxId>) -> anyhow::Result<()> {
        let mut elapsed = 0;
        for ue_tx_id in ue_tx_ids {
            self.check_ue_tx_exist_in_pool(&mut elapsed, ue_tx_id)
                .await?;
        }
        Ok(())
    }

    async fn check_ue_tx_exist_in_pool(
        &self,
        elapsed: &mut u64,
        ue_tx_id: TxId,
    ) -> anyhow::Result<()> {
        while !self
            .blockchain_db
            .ue_tx_exists_in_pool(ue_tx_id.clone())
            .await?
        {
            // tx not exists
            if *elapsed < 10_000 {
                const MILLIS: u64 = 100;
                tokio::time::sleep(Duration::from_millis(MILLIS)).await;
                *elapsed += MILLIS;
            } else {
                return Err(anyhow!("update entity tx {} not exist.", ue_tx_id));
            }
        }
        Ok(())
    }

    fn tx_keys_of(block: &Block) -> (Vec<TxId>, Vec<TxId>, Vec<WasmId>) {
        (
            block
                .act_txs
                .iter()
                .map(|act_tx| act_tx.calc_hash())
                .collect(),
            block.ue_txs.iter().map(|ue_tx| ue_tx.calc_hash()).collect(),
            block
                .wasm_txs
                .iter()
                .map(|wasm_tx| wasm_tx.wasm_id.clone())
                .collect(),
        )
    }

    fn block_state(prev_block: &BlockInDb, act_txs: &[ActTx]) -> BlockState {
        BlockState {
            total_act_txs: prev_block.state.total_act_txs + act_txs.len() as u64,
        }
    }

    fn calc_block_hash(
        height: BlockHeight,
        timestamp: Timestamp,
        state: &BlockState,
        act_tx_ids: &[TxId],
        ue_tx_ids: &[TxId],
        wasm_txs: &[WasmTx],
        prev_hash: &BlockHash,
    ) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(height.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(state.total_act_txs.to_be_bytes());
        for act_tx_id in act_tx_ids {
            hasher.update(act_tx_id);
        }
        for ue_tx_id in ue_tx_ids {
            hasher.update(ue_tx_id);
        }
        for wasm_tx in wasm_txs {
            hasher.update(&wasm_tx.wasm_id.proto);
            hasher.update(&wasm_tx.wasm_id.wasm_hash);
            hasher.update(wasm_tx.wasm_info.block_interval.to_be_bytes());
        }
        hasher.update(prev_hash);
        hasher.into()
    }

    async fn try_insert_download_wasm_tasks(&self, wasm_ids: &[WasmId]) {
        for wasm_id in wasm_ids {
            match self
                .wasm_db
                .try_insert_download_wasm_task(wasm_id.wasm_hash.clone())
                .await
            {
                Ok(_) => {
                    ServiceStarter::new(DownloadWasmTask::new(wasm_id.wasm_hash.clone())).start();
                }
                Err(err) => {
                    log::error!(
                        "try_insert_download_wasm_task {} err: {:?}",
                        wasm_id.wasm_hash,
                        err
                    );
                }
            }
        }
    }
}
