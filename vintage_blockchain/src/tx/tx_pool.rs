use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Mutex, MutexGuard};
use vintage_msg::{ActTx, WasmId, WasmInfo};
use vintage_utils::Hashed;

pub(crate) type TxId = Hashed;

pub(crate) struct TxPool {
    act_txs: Mutex<HashMap<TxId, ActTx>>,
    wasm_txs: Mutex<HashMap<WasmId, WasmInfo>>,
}

impl TxPool {
    pub fn new(act_capacity: usize, wasm_capacity: usize) -> Self {
        Self {
            act_txs: Mutex::new(HashMap::with_capacity(act_capacity)),
            wasm_txs: Mutex::new(HashMap::with_capacity(wasm_capacity)),
        }
    }

    pub fn act_txs_guard(&self) -> MutexGuard<'_, HashMap<TxId, ActTx>> {
        self.act_txs.lock().unwrap()
    }

    pub fn wasm_txs_guard(&self) -> MutexGuard<'_, HashMap<WasmId, WasmInfo>> {
        self.wasm_txs.lock().unwrap()
    }
}

pub fn remove_txs_from_pool<TTxId, TTx>(pool: &mut HashMap<TTxId, TTx>, tx_ids: &[TTxId])
where
    TTxId: Hash + Eq,
{
    for tx_id in tx_ids {
        pool.remove(tx_id);
    }
}
