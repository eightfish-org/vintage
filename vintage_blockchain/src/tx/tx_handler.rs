use crate::db::DbRead;
use anyhow::anyhow;
use redb::Database;
use tokio::sync::mpsc;
use vintage_msg::{NetworkMsg, Tx, TxId};
use vintage_utils::{Pool, SendMsg};

pub(crate) type TxPool = Pool<TxId, Tx>;

pub(crate) fn raw_tx_handler(
    db: &Database,
    tx_pool: &mut TxPool,
    network_msg_sender: &mpsc::Sender<NetworkMsg>,
    tx: Tx,
) {
    match check_tx_not_exist(db, tx_pool, &tx) {
        Ok(_) => {
            tx_pool.insert(tx.clone());
            network_msg_sender.send_msg(NetworkMsg::BroadcastTx(tx));
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }
}

pub(crate) fn tx_handler(db: &Database, tx_pool: &mut TxPool, tx: Tx) {
    match check_tx_not_exist(db, tx_pool, &tx) {
        Ok(_) => {
            tx_pool.insert(tx);
        }
        Err(err) => {
            log::error!("{}", err);
        }
    }
}

fn check_tx_not_exist(db: &Database, tx_pool: &TxPool, tx: &Tx) -> anyhow::Result<()> {
    if tx_pool.contains_id(&tx.id) {
        return Err(anyhow!("tx already exists in pool"));
    }

    let transaction = db.begin_read()?;
    DbRead::check_tx_not_exists(&transaction, tx.id)?;

    Ok(())
}
