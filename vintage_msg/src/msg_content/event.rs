use crate::{ActTx, BlockHeight, EntityId, Model, Proto, ReqId};
use vintage_utils::{Hashed, Timestamp};

pub struct BlockEvent {
    pub height: BlockHeight,
    pub timestamp: Timestamp,
    pub act_events: Vec<ActEvent>,
    pub ue_events: Vec<UpdateEntityEvent>,
}

pub struct ActEvent {
    pub act_tx: ActTx,
    pub act_number: u64,
    pub random: Hashed,
}

pub struct UpdateEntityEvent {
    pub proto: Proto,
    pub model: Model,
    pub req_id: ReqId,
    pub entity_ids: Vec<EntityId>,
}
