use tokio::sync::mpsc;
use vintage_msg::{
    ActEvent, ActTx, BlockEvent, BlockHeight, MsgToProxy, UpdateEntityEvent, UpdateEntityTx,
    WasmHash, WasmId,
};
use vintage_utils::{CalcHash, SendMsg, Timestamp};

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
        timestamp: Timestamp,
        total_act_txs: u64,
        act_txs: Vec<ActTx>,
        ue_txs: Vec<UpdateEntityTx>,
        upgrade_wasm_ids: Vec<WasmId>,
    ) -> bool {
        self.sender
            .send_msg(MsgToProxy::BlockEvent(Self::block_event(
                height,
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
        timestamp: Timestamp,
        total_act_txs: u64,
        act_txs: Vec<ActTx>,
        ue_txs: Vec<UpdateEntityTx>,
        upgrade_wasm_ids: Vec<WasmId>,
    ) -> BlockEvent {
        let mut act_number = total_act_txs - act_txs.len() as u64;
        let mut act_events = Vec::new();
        for act_tx in act_txs {
            act_number += 1;
            act_events.push(ActEvent {
                act_tx,
                act_number,
                random: act_number.calc_hash(),
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
            timestamp,
            act_events,
            ue_events,
            upgrade_wasm_ids,
        }
    }
}
