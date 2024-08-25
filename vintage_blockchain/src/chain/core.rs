use crate::chain::BlockState;
use crate::db::{BlockChainDb, BlockInDb};
use crate::proxy::MsgToProxySender;
use crate::tx::{TxId, TxPool};
use crate::{MAX_ACT_COUNT_PER_BLOCK, MAX_UE_TX_COUNT_PER_BLOCK};
use anyhow::anyhow;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use vintage_msg::{Act, Block, BlockHash, BlockHeight, CalcHash, Hashed};
use vintage_utils::{current_timestamp, Timestamp};

pub type ArcBlockChainCore = Arc<tokio::sync::Mutex<BlockChainCore>>;

pub(crate) struct BlockChainCore {
    db: BlockChainDb,
    tx_pool: Arc<TxPool>,
    proxy_msg_sender: MsgToProxySender,
}

impl BlockChainCore {
    pub fn new(db: BlockChainDb, tx_pool: Arc<TxPool>, proxy_msg_sender: MsgToProxySender) -> Self {
        Self {
            db,
            tx_pool,
            proxy_msg_sender,
        }
    }
}

impl BlockChainCore {
    pub async fn get_block_height(&self) -> anyhow::Result<BlockHeight> {
        self.db.get_block_height().await
    }

    pub async fn new_block(&self, height: u64) -> anyhow::Result<(Block, Hashed)> {
        self.check_block_height(height).await?;
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_ids, acts) = { self.tx_pool.guard().get_acts(MAX_ACT_COUNT_PER_BLOCK) };
        let (ue_tx_ids, ue_txs) = self
            .db
            .get_ue_txs_in_pool(MAX_UE_TX_COUNT_PER_BLOCK)
            .await?;

        // state
        let state = Self::block_state(&prev_block, &acts);

        // hash
        let timestamp = current_timestamp();
        let hash = Self::calc_block_hash(
            height,
            timestamp,
            &state,
            &act_ids,
            &ue_tx_ids,
            &prev_block.hash,
        );

        // new block
        Ok((
            Block {
                timestamp,
                acts,
                ue_txs,
            },
            hash,
        ))
    }

    pub async fn check_block(&self, height: u64, block: Block, hash: Hashed) -> anyhow::Result<()> {
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_ids, ue_tx_ids) = Self::tx_ids_of(&block);
        self.db.check_acts_not_exist(act_ids.clone()).await?;
        self.db.check_ue_txs_not_exist(ue_tx_ids.clone()).await?;
        self.check_ue_txs_exist_in_pool(ue_tx_ids.clone()).await?;

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // hash
        let calc_hash = Self::calc_block_hash(
            height,
            block.timestamp,
            &state,
            &act_ids,
            &ue_tx_ids,
            &prev_block.hash,
        );
        if hash == calc_hash {
            Ok(())
        } else {
            Err(anyhow!("block hash, {} != {}", hash, calc_hash).into())
        }
    }

    pub async fn commit_block(
        &mut self,
        height: u64,
        block: Block,
        hash: Hashed,
    ) -> anyhow::Result<()> {
        self.check_block_height(height).await?;
        // prev block
        let prev_block = self.get_block(height - 1).await?;

        // tx
        let (act_ids, ue_tx_ids) = Self::tx_ids_of(&block);
        let acts = block.acts.clone();
        let ue_txs = block.ue_txs.clone();

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // commit block
        let block_hash_cloned = hash.clone();
        let timestamp = block.timestamp;
        let total_acts = state.total_acts;
        self.db
            .commit_block(height, hash, state, act_ids.clone(), ue_tx_ids, block)
            .await?;
        log::info!(
            "block commited, height: {},\nhash: {}, timestamp: {}, total_acts: {}, acts: {}, ue_txs: {}",
            height,
            block_hash_cloned,
            timestamp,
            total_acts,
            acts.len(),
            ue_txs.len()
        );

        // after - commit block
        {
            self.tx_pool.guard().remove_acts(&act_ids);
        }
        self.proxy_msg_sender
            .send_block_event(timestamp, total_acts, acts, ue_txs);

        Ok(())
    }
}

impl BlockChainCore {
    async fn check_block_height(&self, height: BlockHeight) -> anyhow::Result<()> {
        let last_height = self.db.get_block_height().await?;
        if height == last_height + 1 {
            Ok(())
        } else {
            Err(anyhow!("height is {}, last height is {}", height, last_height).into())
        }
    }

    async fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        self.db.get_block(height).await
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
        while !self.db.ue_tx_exists_in_pool(ue_tx_id.clone()).await? {
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

    fn tx_ids_of(block: &Block) -> (Vec<TxId>, Vec<TxId>) {
        (
            block.acts.iter().map(|act| act.calc_hash()).collect(),
            block.ue_txs.iter().map(|act| act.calc_hash()).collect(),
        )
    }

    fn block_state(prev_block: &BlockInDb, acts: &Vec<Act>) -> BlockState {
        BlockState {
            total_acts: prev_block.state.total_acts + acts.len() as u64,
        }
    }

    fn calc_block_hash(
        height: BlockHeight,
        timestamp: Timestamp,
        state: &BlockState,
        act_ids: &Vec<TxId>,
        ue_tx_ids: &Vec<TxId>,
        prev_hash: &BlockHash,
    ) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(height.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(state.total_acts.to_be_bytes());
        for act_id in act_ids {
            hasher.update(act_id);
        }
        for ue_tx_id in ue_tx_ids {
            hasher.update(ue_tx_id);
        }
        hasher.update(prev_hash);
        hasher.into()
    }
}
