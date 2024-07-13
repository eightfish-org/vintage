use crate::db::Db;
use anyhow::anyhow;
use tokio::sync::mpsc;
use vintage_msg::{NetworkMsg, Tx};
use vintage_utils::{Pool, SendMsg};

pub(crate) type TxPool = Pool<Tx>;

pub(crate) fn raw_tx_handler(
    db: &Db,
    tx_pool: &mut TxPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    tx: Tx,
) -> anyhow::Result<()> {
    check_tx_not_exist(db, tx_pool, &tx)?;
    tx_pool.insert(tx.clone());
    network_msg_sender.send_msg(NetworkMsg::BroadcastTx(tx));
    Ok(())
}

pub(crate) fn tx_handler(db: &Db, tx_pool: &mut TxPool, tx: Tx) -> anyhow::Result<()> {
    check_tx_not_exist(db, tx_pool, &tx)?;
    tx_pool.insert(tx);
    Ok(())
}

fn check_tx_not_exist(db: &Db, tx_pool: &TxPool, tx: &Tx) -> anyhow::Result<()> {
    if tx_pool.contains_id(&tx.id) {
        return Err(anyhow!("tx already exists in pool"));
    }
    db.begin_read()?.open_txs()?.check_tx_not_exists(tx.id)?;
    Ok(())
}
