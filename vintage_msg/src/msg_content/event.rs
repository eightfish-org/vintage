use crate::{Act, EntityId, Hashed, Model, ReqId};
use vintage_utils::Timestamp;

pub struct BlockEvent {
    pub timestamp: Timestamp,
    pub act_events: Vec<ActEvent>,
    pub ue_events: Vec<UpdateEntityEvent>,
}

pub struct ActEvent {
    pub act: Act,
    pub nonce: u64,
    pub random: Hashed,
}

pub struct UpdateEntityEvent {
    pub model: Model,
    pub req_id: ReqId,
    pub entity_ids: Vec<EntityId>,
}
