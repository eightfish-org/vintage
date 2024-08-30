use rand::{random, thread_rng, Rng};
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_msg::{Act, MsgToBlockChain};
use vintage_utils::SendMsg;

pub(super) async fn send_raw_act_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(2000..=3000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::Act(random_act()));
    }
}

pub(super) async fn send_act_to_blockchain(sender: mpsc::Sender<MsgToBlockChain>) {
    loop {
        let millis = thread_rng().gen_range(500..=1000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(MsgToBlockChain::Broadcast(
            serde_json::to_vec(&random_act()).unwrap(),
        ));
    }
}

pub(super) async fn send_block_to_blockchain(_sender: mpsc::Sender<MsgToBlockChain>) {
    let mut _height_block = 1;

    loop {
        let _ordered = thread_rng().gen_range(1..=100) <= 70; // 70%概率是顺序的

        let millis = thread_rng().gen_range(1000..=2000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        // sender.send_msg(BlockChainMsg::ProduceBlock(BlockProduction {
        //     block_height: if ordered {
        //         height_block
        //     } else {
        //         height_block + 1
        //     },
        // }));

        let millis = thread_rng().gen_range(1000..=2000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        // sender.send_msg(BlockChainMsg::ProduceBlock(BlockProduction {
        //     block_height: if ordered {
        //         height_block + 1
        //     } else {
        //         height_block
        //     },
        // }));

        _height_block += 2;
    }
}

fn random_act() -> Act {
    let len = thread_rng().gen_range(100..=1000);
    let mut content = Vec::<u8>::with_capacity(len);
    for _ in 0..len {
        content.push(random())
    }

    Act {
        kind: "post".to_owned(),
        model: "model1".to_owned(),
        data: Vec::new(),
    }
}
