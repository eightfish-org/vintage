use crate::tx::TxId;
use std::collections::HashMap;
use vintage_msg::ActTx;

pub fn get_act_txs_from_pool(pool: &HashMap<TxId, ActTx>, count: usize) -> (Vec<TxId>, Vec<ActTx>) {
    let mut act_tx_ids = Vec::new();
    let mut act_txs = Vec::new();
    for (hash, act_tx) in pool.iter().take(count) {
        act_tx_ids.push(hash.clone());
        act_txs.push(act_tx.clone());
    }
    (act_tx_ids, act_txs)
}
