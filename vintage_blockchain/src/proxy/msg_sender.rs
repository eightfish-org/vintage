use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use vintage_msg::{
    ActEvent, ActTx, BlockEvent, BlockHash, BlockHeight, MsgToProxy, UpdateEntityEvent,
    UpdateEntityTx, WasmHash, WasmId,
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
        upgrade_wasm_ids: Vec<WasmId>,
    ) -> bool {
        self.sender
            .send_msg(MsgToProxy::BlockEvent(Self::block_event(
                height,
                block_hash,
                timestamp,
                total_act_txs,
                act_txs,
                ue_txs,
                upgrade_wasm_ids,
            )))
    }

    pub fn send_wasm_binary(&self, wasm_hash: WasmHash, wash_binary: Vec<u8>) -> bool {
        self.sender
            .send_msg(MsgToProxy::WasmBinary(wasm_hash, wash_binary))
    }
}

impl MsgToProxySender {
    fn block_event(
        height: BlockHeight,
        block_hash: BlockHash,
        timestamp: Timestamp,
        _total_act_txs: u64,
        act_txs: Vec<ActTx>,
        ue_txs: Vec<UpdateEntityTx>,
        upgrade_wasm_ids: Vec<WasmId>,
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
                proto: ue_tx.proto,
                model: ue_tx.model,
                req_id: ue_tx.req_id,
                entity_ids: ue_tx.entities.into_iter().map(|entity| entity.id).collect(),
            })
        }

        BlockEvent {
            height,
            block_hash,
            timestamp,
            act_events,
            ue_events,
            upgrade_wasm_ids,
        }
    }
}

fn calc_act_random(block_hash: &BlockHash, act_number: u64) -> Hashed {
    let mut hasher = Sha256::new();
    hasher.update(block_hash);
    hasher.update(act_number.to_be_bytes());
    hasher.into()
}
