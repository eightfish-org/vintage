use crate::db::AsyncBlockChainDb;
use anyhow::anyhow;
use tokio::sync::mpsc;
use vintage_msg::{NetworkMsg, Tx};
use vintage_utils::{Pool, SendMsg};

pub(crate) type TxPool = Pool<Tx>;

pub(crate) async fn raw_tx_handler(
    db: &AsyncBlockChainDb,
    tx_pool: &mut TxPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    tx: Tx,
) -> anyhow::Result<()> {
    check_tx_not_exists(db, tx_pool, &tx).await?;
    log::info!(
        "tx from worker: {:032X}, content len: {}",
        tx.id,
        tx.content.len()
    );
    tx_pool.insert(tx.clone());

    network_msg_sender.send_msg(NetworkMsg::BroadcastTx(tx));
    Ok(())
}

pub(crate) async fn tx_handler(
    db: &AsyncBlockChainDb,
    tx_pool: &mut TxPool,
    tx: Tx,
) -> anyhow::Result<()> {
    check_tx_not_exists(db, tx_pool, &tx).await?;
    log::info!(
        "tx from network: {:032X}, content len: {}",
        tx.id,
        tx.content.len()
    );
    tx_pool.insert(tx);

    Ok(())
}

async fn check_tx_not_exists(
    db: &AsyncBlockChainDb,
    tx_pool: &TxPool,
    tx: &Tx,
) -> anyhow::Result<()> {
    if tx_pool.contains_id(&tx.id) {
        return Err(anyhow!("tx already exists in pool"));
    }
    db.check_tx_not_exists(tx.id).await?;
    Ok(())
}
