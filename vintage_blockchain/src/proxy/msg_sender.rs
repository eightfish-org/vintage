use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use vintage_msg::{
    ActEvent, ActTx, BlockEvent, BlockHash, BlockHeight, EntityKey, MsgToProxy, UpdateEntityEvent,
    UpdateEntityTx, UpgradeWasmEvent, UploadWasmEvent, WasmHash, WasmTx,
};
use vintage_utils::{Hashed, SendMsg, Timestamp};

#[derive(Clone)]
pub(crate) struct MsgToProxySender {
    sender: mpsc::Sender<MsgToProxy>,
}

impl MsgToProxySender {
    pub fn new(sender: mpsc::Sender<MsgToProxy>) -> Self {
        Self { sender }
    }

    pub fn send_block_event(
        &self,
        height: BlockHeight,
        block_hash: BlockHash,
        timestamp: Timestamp,
        total_act_txs: u64,
        act_txs: Vec<ActTx>,
        ue_txs: Vec<UpdateEntityTx>,
        upgrade_wasm_txs: Vec<WasmTx>,
    ) -> bool {
        self.sender.send_msg(MsgToProxy::BlockEvent(block_event(
            height,
            block_hash,
            timestamp,
            total_act_txs,
            act_txs,
            ue_txs,
            upgrade_wasm_txs,
        )))
    }

    pub fn send_upload_wasm_event(&self, wasm_hash: WasmHash, wasm_binary: Vec<u8>) -> bool {
        self.sender
            .send_msg(MsgToProxy::UploadWasmEvent(UploadWasmEvent {
                wasm_hash,
                wasm_binary,
            }))
    }
}

fn block_event(
    height: BlockHeight,
    block_hash: BlockHash,
    timestamp: Timestamp,
    _total_act_txs: u64,
    act_txs: Vec<ActTx>,
    ue_txs: Vec<UpdateEntityTx>,
    upgrade_wasm_txs: Vec<WasmTx>,
) -> BlockEvent {
    // let mut act_number = total_act_txs - act_txs.len() as u64;
    let mut act_number = 0;
    let mut act_events = Vec::new();
    for act_tx in act_txs {
        act_number += 1;
        act_events.push(ActEvent {
            act_tx,
            act_number,
            random: calc_act_random(&block_hash, act_number),
        })
    }

    let mut ue_events = Vec::new();
    for ue_tx in ue_txs {
        ue_events.push(UpdateEntityEvent {
            req_id: ue_tx.req_id,
            proto: ue_tx.proto,
            entity_keys: ue_tx
                .entities
                .into_iter()
                .map(|entity| EntityKey {
                    model: entity.model,
                    id: entity.id,
                })
                .collect(),
        })
    }

    let mut upgrade_wasm_events = Vec::new();
    for upgrade_wasm_tx in upgrade_wasm_txs {
        upgrade_wasm_events.push(UpgradeWasmEvent {
            proto: upgrade_wasm_tx.proto,
            wasm_hash: upgrade_wasm_tx.wasm_hash,
            sql: upgrade_wasm_tx.sql,
        })
    }

    BlockEvent {
        height,
        block_hash,
        timestamp,
        act_events,
        ue_events,
        upgrade_wasm_events,
    }
}

fn calc_act_random(block_hash: &BlockHash, act_number: u64) -> Hashed {
    let mut hasher = Sha256::new();
    hasher.update(block_hash);
    hasher.update(act_number.to_be_bytes());
    hasher.into()
}
