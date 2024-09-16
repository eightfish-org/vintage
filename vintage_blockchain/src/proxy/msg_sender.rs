use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use vintage_msg::{
    Act, ActEvent, BlockEvent, BlockHeight, Hashed, MsgToProxy, UpdateEntityEvent, UpdateEntityTx,
};
use vintage_utils::{SendMsg, Timestamp};

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
        total_acts: u64,
        acts: Vec<Act>,
        ue_txs: Vec<UpdateEntityTx>,
    ) -> bool {
        self.sender
            .send_msg(MsgToProxy::BlockEvent(Self::block_event(
                height, timestamp, total_acts, acts, ue_txs,
            )))
    }
}

impl MsgToProxySender {
    fn block_event(
        height: BlockHeight,
        timestamp: Timestamp,
        total_acts: u64,
        acts: Vec<Act>,
        ue_txs: Vec<UpdateEntityTx>,
    ) -> BlockEvent {
        let mut act_number = total_acts - acts.len() as u64;
        let mut act_events = Vec::new();
        for act in acts {
            act_number += 1;
            act_events.push(ActEvent {
                act,
                act_number,
                random: Self::random(act_number),
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
        }
    }

    fn random(nonce: u64) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(nonce.to_be_bytes());
        hasher.into()
    }
}
