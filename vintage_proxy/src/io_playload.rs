use serde::{Deserialize, Serialize};
use serde_json::json;
use vintage_msg::{EntityHash, EntityId, ReqId};

pub(crate) fn req_payload_json<TReqData>(req_id: &ReqId, req_data: TReqData) -> serde_json::Value
where
    TReqData: Serialize,
{
    json!({
        "reqid": req_id,
        "reqdata": req_data,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReqPayload<TReqData> {
    pub reqid: ReqId,
    pub reqdata: TReqData,
}

pub(crate) type EntitiesReqPayload = ReqPayload<Vec<(EntityId, EntityHash)>>;
