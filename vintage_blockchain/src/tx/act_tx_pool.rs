use std::collections::HashMap;
use vintage_msg::{ActTx, TxId};

pub fn get_act_txs_from_pool(pool: &HashMap<TxId, ActTx>, count: usize) -> (Vec<TxId>, Vec<ActTx>) {
    let mut tx_ids = Vec::new();
    let mut txs = Vec::new();
    for (tx_id, tx) in pool.iter().take(count) {
        tx_ids.push(tx_id.clone());
        txs.push(tx.clone());
    }
    (tx_ids, txs)
}
