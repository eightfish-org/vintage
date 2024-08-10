use crate::chain::BlockState;
use crate::db::{BlockChainDb, BlockInDb};
use crate::tx::{TxId, TxPool};
use crate::{MAX_ACT_COUNT_PER_BLOCK, MAX_UE_TX_COUNT_PER_BLOCK};
use anyhow::anyhow;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_consensus::BlockConsensus;
use vintage_msg::{
    Act, ActEvent, Block, BlockEvent, BlockHash, BlockHeight, CalcHash, Hash, Hashed, ProxyMsg,
    UpdateEntityEvent, UpdateEntityTx,
};
use vintage_utils::{current_timestamp, SendMsg, Timestamp};

pub struct BlockChain {
    db: BlockChainDb,
    tx_pool: Arc<TxPool>,
    proxy_msg_sender: mpsc::Sender<ProxyMsg>,
}

impl BlockChain {
    pub(crate) fn new(
        db: BlockChainDb,
        tx_pool: Arc<TxPool>,
        proxy_msg_sender: mpsc::Sender<ProxyMsg>,
    ) -> Self {
        Self {
            db,
            tx_pool,
            proxy_msg_sender,
        }
    }
}

#[async_trait]
impl BlockConsensus<Block> for BlockChain {
    async fn get_block(&self, height: u64) -> Result<(Block, Hash), Box<dyn Error + Send>> {
        // prev block
        self.check_block_height(height).await?;
        let prev_block = self.get_pre_block(height).await?;

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
            (&hash).into(),
        ))
    }

    async fn check_block(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        // prev block
        let prev_block = self.get_pre_block(height).await?;

        // tx
        let (act_ids, ue_tx_ids) = Self::tx_ids_of(&block);
        self.db.check_acts_not_exist(act_ids.clone()).await?;
        self.db.check_ue_txs_not_exist(ue_tx_ids.clone()).await?;
        self.check_ue_txs_exist_in_pool(ue_tx_ids.clone()).await?;

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // hash
        let block_hash: BlockHash = (&hash).try_into()?;
        let calc_hash = Self::calc_block_hash(
            height,
            block.timestamp,
            &state,
            &act_ids,
            &ue_tx_ids,
            &prev_block.hash,
        );
        if block_hash == calc_hash {
            Ok(())
        } else {
            Err(anyhow!("block hash, {} != {}", block_hash, calc_hash).into())
        }
    }

    async fn commit(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        // prev block
        self.check_block_height(height).await?;
        let prev_block = self.get_pre_block(height).await?;

        // tx
        let (act_ids, ue_tx_ids) = Self::tx_ids_of(&block);
        let acts = block.acts.clone();
        let ue_txs = block.ue_txs.clone();

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // hash
        let block_hash: BlockHash = (&hash).try_into()?;

        // commit block
        let timestamp = block.timestamp;
        let total_acts = state.total_acts;
        self.db
            .commit_block(height, block_hash, state, act_ids.clone(), ue_tx_ids, block)
            .await?;
        log::info!("block {} commited, act count: {}", height, acts.len());

        // after - commit block
        {
            self.tx_pool.guard().remove_acts(&act_ids);
        }
        self.proxy_msg_sender
            .send_msg(ProxyMsg::BlockEvent(Self::block_event(
                timestamp, total_acts, acts, ue_txs,
            )));

        Ok(())
    }
}

impl BlockChain {
    async fn check_block_height(&self, height: BlockHeight) -> anyhow::Result<()> {
        let last_height = self.db.get_last_block_height().await?;
        if height == last_height + 1 {
            Ok(())
        } else {
            Err(anyhow!("height is {}, last height is {}", height, last_height).into())
        }
    }

    async fn get_pre_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        self.db.get_block(height - 1).await
    }

    async fn check_ue_txs_exist_in_pool(&self, ue_tx_ids: Vec<TxId>) -> anyhow::Result<()> {
        let mut count = 0;
        for ue_tx_id in ue_tx_ids {
            self.check_ue_tx_exist_in_pool(&mut count, ue_tx_id).await?;
        }
        Ok(())
    }

    async fn check_ue_tx_exist_in_pool(
        &self,
        count: &mut u32,
        ue_tx_id: TxId,
    ) -> anyhow::Result<()> {
        while !self.db.ue_tx_exists_in_pool(ue_tx_id.clone()).await? {
            // tx not exists
            // 100ms * 300 = 30s
            if *count < 300u32 {
                *count += 1;
                tokio::time::sleep(Duration::from_millis(100)).await;
            } else {
                return Err(anyhow!("update entity tx {} not exist.", ue_tx_id));
            }
        }
        Ok(())
    }
}

impl BlockChain {
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

    fn block_event(
        timestamp: Timestamp,
        total_acts: u64,
        acts: Vec<Act>,
        ue_txs: Vec<UpdateEntityTx>,
    ) -> BlockEvent {
        let mut nonce = total_acts - acts.len() as u64;
        let mut act_events = Vec::new();
        for act in acts {
            nonce += 1;
            act_events.push(ActEvent {
                act,
                nonce,
                random: Self::random(nonce),
            })
        }

        let mut ue_events = Vec::new();
        for ue_tx in ue_txs {
            ue_events.push(UpdateEntityEvent {
                model: ue_tx.model,
                req_id: ue_tx.req_id,
                entity_ids: ue_tx.entities.into_iter().map(|entity| entity.id).collect(),
            })
        }

        BlockEvent {
            timestamp,
            act_events,
            ue_events,
        }
    }

    fn random(nonce: u64) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(nonce.to_be_bytes());
        hasher.into()
    }
}
