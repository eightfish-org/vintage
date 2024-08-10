use crate::Hashed;
use serde::{Deserialize, Serialize};

pub type Model = String;

////////////////////////////////////////////////////////////////////////////////////////////////////
// act

pub type ActKind = String;
pub type ActData = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Act {
    pub kind: ActKind,
    pub model: Model,
    pub data: ActData,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// entity

pub type EntityId = String;
pub type EntityHash = Hashed;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub hash: EntityHash,
}

pub type ReqId = String;

pub struct UpdateEntities {
    pub model: Model,
    pub req_id: ReqId,
    pub entities: Vec<Entity>,
}
