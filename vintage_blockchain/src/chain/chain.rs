use crate::chain::BlockState;
use crate::db::{BlockChainDb, BlockInDb};
use crate::tx::{calc_act_id, ActId, TxPool};
use crate::MAX_ACT_COUNT_PER_BLOCK;
use anyhow::anyhow;
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc;
use vintage_consensus::BlockConsensus;
use vintage_msg::{
    Act, ActEvent, Block, BlockEvent, BlockHash, BlockHeight, Hash, Hashed, ProxyMsg,
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

        // acts
        let (act_ids, acts) = { self.tx_pool.guard().acts.get_acts(MAX_ACT_COUNT_PER_BLOCK) };

        // state
        let state = Self::block_state(&prev_block, &acts);

        // hash
        let timestamp = current_timestamp();
        let hash = Self::calc_block_hash(height, timestamp, &state, &act_ids, &prev_block.hash);

        // new block
        Ok((Block { timestamp, acts }, (&hash).into()))
    }

    async fn check_block(
        &self,
        height: u64,
        block: Block,
        hash: Hash,
    ) -> Result<(), Box<dyn Error + Send>> {
        // prev block
        let prev_block = self.get_pre_block(height).await?;

        // acts
        let act_ids = Self::act_ids_of(&block);
        self.db.check_acts_not_exist(act_ids.clone()).await?;

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // hash
        let block_hash: BlockHash = (&hash).try_into()?;
        let calc_hash =
            Self::calc_block_hash(height, block.timestamp, &state, &act_ids, &prev_block.hash);
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

        // acts
        let act_ids = Self::act_ids_of(&block);
        let acts = block.acts.clone();

        // state
        let state = Self::block_state(&prev_block, &block.acts);

        // hash
        let block_hash: BlockHash = (&hash).try_into()?;

        // commit block
        let timestamp = block.timestamp;
        let total_acts = state.total_acts;
        self.db
            .commit_block(height, block_hash, state, act_ids.clone(), block)
            .await?;
        log::info!("block {} commited, act count: {}", height, act_ids.len());

        // after - commit block
        {
            self.tx_pool.guard().acts.remove_acts(&act_ids);
        }
        self.proxy_msg_sender
            .send_msg(ProxyMsg::BlockEvent(Self::block_event(
                timestamp, total_acts, acts,
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

    fn act_ids_of(block: &Block) -> Vec<ActId> {
        block.acts.iter().map(|act| calc_act_id(act)).collect()
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
        act_ids: &Vec<ActId>,
        prev_hash: &BlockHash,
    ) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(height.to_be_bytes());
        hasher.update(timestamp.to_be_bytes());
        hasher.update(state.total_acts.to_be_bytes());
        for act_id in act_ids {
            hasher.update(act_id);
        }
        hasher.update(prev_hash);
        BlockHash::new(hasher.finalize().into())
    }

    fn block_event(timestamp: Timestamp, total_acts: u64, act_vec: Vec<Act>) -> BlockEvent {
        let mut nonce = total_acts - act_vec.len() as u64;
        let mut acts = Vec::new();
        for act in act_vec {
            nonce += 1;
            acts.push(ActEvent {
                act,
                nonce,
                random: {
                    let mut hasher = Sha256::new();
                    hasher.update(nonce.to_be_bytes());
                    Hashed::new(hasher.finalize().into())
                },
            })
        }
        BlockEvent {
            timestamp,
            acts,
            entities: vec![],
        }
    }
}
