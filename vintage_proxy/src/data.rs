use serde::{Deserialize, Serialize};
use serde_json::json;
use vintage_msg::{ActionKind, EntityHash, EntityId, Model, ReqId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputOutputObject {
    pub action: ActionKind,
    pub model: Model,
    pub data: Vec<u8>,
    pub ext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Payload<TReqData> {
    pub reqid: ReqId,
    pub reqdata: TReqData,
}

pub(crate) type EntitiesPayload = Payload<Vec<(EntityId, EntityHash)>>;

pub fn payload_json(req_id: &ReqId, req_data: impl Serialize) -> serde_json::Value {
    json!({
        "reqid": req_id,
        "reqdata": req_data,
    })
}
