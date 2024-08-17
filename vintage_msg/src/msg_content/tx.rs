use crate::{CalcHash, Hashed};
use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub type ActionKind = String;
pub type Model = String;

////////////////////////////////////////////////////////////////////////////////////////////////////
// act

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Act {
    pub kind: ActionKind,
    pub model: Model,
    pub data: Vec<u8>,
}

impl CalcHash for Act {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(&self.kind);
        hasher.update(&self.model);
        hasher.update(&self.data);
        hasher.into()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// entity

pub type EntityId = String;
pub type EntityHash = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub hash: EntityHash,
}

pub type ReqId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateEntityTx {
    pub model: Model,
    pub req_id: ReqId,
    pub entities: Vec<Entity>,
}

impl CalcHash for UpdateEntityTx {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(&self.model);
        hasher.update(&self.req_id);
        for entity in &self.entities {
            hasher.update(&entity.id);
            hasher.update(&entity.hash);
        }
        hasher.into()
    }
}
