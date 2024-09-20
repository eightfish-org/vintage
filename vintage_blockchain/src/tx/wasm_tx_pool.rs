use std::collections::HashMap;
use vintage_msg::{WasmId, WasmInfo, WasmTx};

pub fn get_wasm_txs_from_pool(pool: &HashMap<WasmId, WasmInfo>) -> Vec<WasmTx> {
    let mut wasm_txs = Vec::new();
    for (wasm_id, wasm_info) in pool {
        wasm_txs.push(WasmTx {
            wasm_id: wasm_id.clone(),
            wasm_info: wasm_info.clone(),
        });
    }
    wasm_txs
}
