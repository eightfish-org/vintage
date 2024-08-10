use crate::{Act, EntityId, Hashed, Model, ReqId};
use vintage_utils::Timestamp;

pub struct BlockEvent {
    pub timestamp: Timestamp,
    pub acts: Vec<ActEvent>,
    pub entities: Vec<EntityEvent>,
}

pub struct ActEvent {
    pub act: Act,
    pub nonce: u64,
    pub random: Hashed,
}

pub struct EntityEvent {
    pub model: Model,
    pub req_id: ReqId,
    pub entity_id: EntityId,
}
