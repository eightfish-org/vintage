use rand::{random, thread_rng, Rng};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_msg::{ActTx, Entity, MsgToBlockChain, UpdateEntityTx, UploadWasm};
use vintage_utils::SendMsg;

pub(super) async fn _broadcast_act_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(500..=1000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::Broadcast(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
            serde_json::to_vec(&random_act_tx()).unwrap(),
        ));
    }
}

pub(super) async fn send_act_tx_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(2000..=3000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::ActTx(random_act_tx()));
    }
}

pub(super) async fn send_ue_tx_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(2000..=3000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::UpdateEntityTx(random_ue_tx()));
    }
}

pub(super) async fn send_wasm_tx_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        tokio::time::sleep(Duration::from_secs(20)).await;
        sender.send_msg(MsgToBlockChain::UploadWasm(UploadWasm {
            proto: "proto_1".to_owned(),
            wasm_binary: random_bytes(),
            sql: format!("this is a sql migration {}", random_string()),
            after_blocks: 10,
        }));
    }
}

fn random_string() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn random_bytes() -> Vec<u8> {
    let len = thread_rng().gen_range(100..=1000);
    let mut data = Vec::<u8>::with_capacity(len);
    for _ in 0..len {
        data.push(random())
    }
    data
}

fn random_act_tx() -> ActTx {
    ActTx {
        action: "post".to_owned(),
        proto: "proto_1".to_owned(),
        model: "model_1".to_owned(),
        data: random_bytes(),
    }
}

fn random_ue_tx() -> UpdateEntityTx {
    UpdateEntityTx {
        proto: "proto_1".to_string(),
        req_id: random_string(),
        entities: vec![Entity {
            model: "model_1".to_string(),
            id: random_string(),
            hash: random_string(),
        }],
    }
}
