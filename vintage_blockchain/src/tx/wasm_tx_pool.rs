use std::collections::HashMap;
use vintage_msg::{TxId, WasmTx};

pub fn get_wasm_txs_from_pool(pool: &HashMap<TxId, WasmTx>) -> (Vec<TxId>, Vec<WasmTx>) {
    let mut tx_ids = Vec::new();
    let mut txs = Vec::new();
    for (tx_id, tx) in pool {
        tx_ids.push(tx_id.clone());
        txs.push(tx.clone());
    }
    (tx_ids, txs)
}
