use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use vintage_utils::{CalcHash, Hashed};

pub type Action = String;
pub type Proto = String;
pub type Model = String;

////////////////////////////////////////////////////////////////////////////////////////////////////
// act

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActTx {
    pub action: Action,
    pub proto: Proto,
    pub model: Model,
    pub data: Vec<u8>,
}

impl CalcHash for ActTx {
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// wasm

pub type WasmHash = Hashed;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmId {
    pub proto: Proto,
    pub wasm_hash: WasmHash,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmInfo {
    pub block_interval: u64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmTx {
    pub wasm_id: WasmId,
    pub wasm_info: WasmInfo,
}
