use rand::{random, thread_rng, Rng};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_msg::{ActTx, MsgToBlockChain, UploadWasm};
use vintage_utils::SendMsg;

pub(super) async fn _broadcast_act_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(500..=1000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::Broadcast(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8000),
            serde_json::to_vec(&random_act()).unwrap(),
        ));
    }
}

pub(super) async fn send_act_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(2000..=3000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::ActTx(random_act()));
    }
}

pub(super) async fn send_wasm_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        tokio::time::sleep(Duration::from_secs(120)).await;
        sender.send_msg(MsgToBlockChain::UploadWasm(UploadWasm {
            proto: "proto2".to_string(),
            wasm_binary: random_bytes(),
            block_interval: 100,
        }));
    }
}

fn random_bytes() -> Vec<u8> {
    let len = thread_rng().gen_range(100..=1000);
    let mut data = Vec::<u8>::with_capacity(len);
    for _ in 0..len {
        data.push(random())
    }
    data
}

fn random_act() -> ActTx {
    ActTx {
        action: "post".to_owned(),
        proto: "proto1".to_owned(),
        model: "model1".to_owned(),
        data: random_bytes(),
    }
}
