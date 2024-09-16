use crate::{CalcHash, Hashed};
use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub type Action = String;
pub type Proto = String;
pub type Model = String;

////////////////////////////////////////////////////////////////////////////////////////////////////
// act

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Act {
    pub action: Action,
    pub proto: Proto,
    pub model: Model,
    pub data: Vec<u8>,
}

impl CalcHash for Act {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(&self.action);
        hasher.update(&self.proto);
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
    pub proto: Proto,
    pub model: Model,
    pub req_id: ReqId,
    pub entities: Vec<Entity>,
}

impl CalcHash for UpdateEntityTx {
    fn calc_hash(&self) -> Hashed {
        let mut hasher = Sha256::new();
        hasher.update(&self.proto);
        hasher.update(&self.model);
        hasher.update(&self.req_id);
        for entity in &self.entities {
            hasher.update(&entity.id);
            hasher.update(&entity.hash);
        }
        hasher.into()
    }
}
