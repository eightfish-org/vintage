use serde::{Deserialize, Serialize};
use vintage_msg::{EntityHash, EntityId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputOutputObject {
    pub action: String,
    pub model: String,
    pub data: Vec<u8>,
    pub ext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Payload<TReqData> {
    pub reqid: String,
    pub reqdata: TReqData,
}

pub(crate) type EntitiesPayload = Payload<Vec<(EntityId, EntityHash)>>;
