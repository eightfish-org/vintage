use crate::{Act, BlockHeight, EntityId, Hashed, Model, Proto, ReqId};
use vintage_utils::Timestamp;

pub struct BlockEvent {
    pub height: BlockHeight,
    pub timestamp: Timestamp,
    pub act_events: Vec<ActEvent>,
    pub ue_events: Vec<UpdateEntityEvent>,
}

pub struct ActEvent {
    pub act: Act,
    pub act_number: u64,
    pub random: Hashed,
}

pub struct UpdateEntityEvent {
    pub proto: Proto,
    pub model: Model,
    pub req_id: ReqId,
    pub entity_ids: Vec<EntityId>,
}
