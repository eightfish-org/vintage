use crate::data::InputOutputObject;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use vintage_msg::{Act, BlockChainMsg, Entity, EntityHash, UpdateEntities};
use vintage_utils::SendMsg;

pub(crate) fn post(sender: &mpsc::Sender<BlockChainMsg>, object: InputOutputObject) {
    sender.send_msg(BlockChainMsg::Act(Act {
        kind: object.action,
        model: object.model,
        data: object.data,
    }))
}

pub(crate) fn update_index(sender: &mpsc::Sender<BlockChainMsg>, object: InputOutputObject) {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct Data {
        pub reqid: String,
        pub reqdata: Option<Vec<(String, EntityHash)>>,
    }
    let data: Data = serde_json::from_slice(&object.data).unwrap();
    println!("from redis: update_index: payload: {:?}", data);

    let entities = data
        .reqdata
        .clone()
        .unwrap()
        .into_iter()
        .map(|(id, hash)| Entity { id, hash })
        .collect();

    sender.send_msg(BlockChainMsg::UpdateEntities(UpdateEntities {
        model: object.model,
        req_id: data.reqid,
        entities,
    }));
}

pub(crate) fn check_pair_list(_object: InputOutputObject) {}

pub(crate) fn check_new_version_wasmfile(_object: InputOutputObject) {}

pub(crate) fn retreive_wasmfile(_object: InputOutputObject) {}

pub(crate) fn disable_wasm_upgrade_flag(_object: InputOutputObject) {}
