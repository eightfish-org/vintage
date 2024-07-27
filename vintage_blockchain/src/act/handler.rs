use crate::db::AsyncBlockChainDb;
use anyhow::anyhow;
use tokio::sync::mpsc;
use vintage_msg::{NetworkMsg, Act};
use vintage_utils::{Pool, SendMsg};

pub(crate) type ActPool = Pool<Act>;

pub(crate) async fn raw_act_handler(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    act: Act,
) -> anyhow::Result<()> {
    check_act_not_exists(db, act_pool, &act).await?;
    log::info!(
        "act from worker: {:032X}, content len: {}",
        act.id,
        act.content.len()
    );
    act_pool.insert(act.clone());

    network_msg_sender.send_msg(NetworkMsg::BroadcastAct(act));
    Ok(())
}

pub(crate) async fn act_handler(
    db: &AsyncBlockChainDb,
    act_pool: &mut ActPool,
    act: Act,
) -> anyhow::Result<()> {
    check_act_not_exists(db, act_pool, &act).await?;
    log::info!(
        "act from network: {:032X}, content len: {}",
        act.id,
        act.content.len()
    );
    act_pool.insert(act);

    Ok(())
}

async fn check_act_not_exists(
    db: &AsyncBlockChainDb,
    act_pool: &ActPool,
    act: &Act,
) -> anyhow::Result<()> {
    if act_pool.contains_id(&act.id) {
        return Err(anyhow!("act already exists in pool"));
    }
    db.check_act_not_exists(act.id).await?;
    Ok(())
}
