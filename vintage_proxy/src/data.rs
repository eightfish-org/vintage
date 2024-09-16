use serde::{Deserialize, Serialize};
use serde_json::json;
use vintage_msg::{Action, EntityHash, EntityId, Model, Proto, ReqId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputOutputObject {
    pub action: Action,
    pub proto: Proto,
    pub model: Model,
    pub data: Vec<u8>,
    pub ext: Vec<u8>,
}

pub(crate) fn payload_json<TReqData>(req_id: &ReqId, req_data: TReqData) -> serde_json::Value
where
    TReqData: Serialize,
{
    json!({
        "reqid": req_id,
        "reqdata": req_data,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Payload<TReqData> {
    pub reqid: ReqId,
    pub reqdata: TReqData,
}

pub(crate) type EntitiesPayload = Payload<Vec<(EntityId, EntityHash)>>;
