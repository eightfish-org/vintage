use crate::block::check_block_hash;
use crate::block::helper::{new_block, persist_block};
use crate::block::{BlockMsg, BlockMsgPool};
use crate::db::AsyncBlockChainDb;
use crate::act::ActPool;
use crate::MAX_ACT_COUNT_PER_BLOCK;
use anyhow::anyhow;
use tokio::sync::mpsc;
use vintage_msg::{Block, BlockProduction, NetworkMsg, ActId};
use vintage_utils::{SendMsg, WithId};

pub(crate) async fn block_msg_handler(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    block_msg_pool: &mut BlockMsgPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    block_msg: BlockMsg,
) -> anyhow::Result<()> {
    let mut next_block_height = db.get_last_block_height().await? + 1;
    let block_height = block_msg.id().clone();

    // check block_height
    if block_height < next_block_height {
        return Err(anyhow!(
            "block_height {} < next_block_height {}",
            block_height,
            next_block_height,
        ));
    } else if block_height > next_block_height {
        log::info!(
            "block_height {} > next_block_height {}",
            block_height,
            next_block_height
        );
        block_msg_pool.insert(block_msg);
        return Ok(());
    }

    block_msg_handler_impl(db, act_pool, network_msg_sender, block_msg).await?;
    next_block_height += 1;

    loop {
        match block_msg_pool.remove(&next_block_height) {
            Some(msg) => {
                block_msg_handler_impl(db, act_pool, network_msg_sender, msg).await?;
                next_block_height += 1;
            }
            None => break,
        }
    }
    Ok(())
}

async fn block_msg_handler_impl(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    msg: BlockMsg,
) -> anyhow::Result<()> {
    match msg {
        BlockMsg::ImportBlock(block) => import_block(db, act_pool, block).await,
        BlockMsg::ProduceBlock(block_production) => {
            produce_block(db, act_pool, network_msg_sender, block_production).await
        }
    }
}

async fn import_block(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    block: Block,
) -> anyhow::Result<()> {
    let act_ids = act_ids_of_block(&block);
    db.check_acts_not_exist(act_ids.clone()).await?;

    let prev_block_hash = db.get_block_hash(block.header.height - 1).await?;
    check_block_hash(&block, &prev_block_hash)?;

    persist_block(db, block.clone()).await?;
    act_pool.remove_by_ids(&act_ids);

    Ok(())
}

async fn produce_block(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    block_production: BlockProduction,
) -> anyhow::Result<()> {
    let prev_block_hash = db.get_block_hash(block_production.block_height - 1).await?;

    let block = new_block(
        block_production.block_height,
        act_pool.get_values(MAX_ACT_COUNT_PER_BLOCK),
        &prev_block_hash,
    );
    let act_ids = act_ids_of_block(&block);

    persist_block(db, block.clone()).await?;
    act_pool.remove_by_ids(&act_ids);

    network_msg_sender.send_msg(NetworkMsg::BroadcastBlock(block));
    Ok(())
}

fn act_ids_of_block(block: &Block) -> Vec<ActId> {
    block.body.acts.iter().map(|act| act.id).collect()
}
