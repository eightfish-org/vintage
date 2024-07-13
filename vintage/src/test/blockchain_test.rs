use rand::{random, thread_rng, Rng};
use std::time::Duration;
use tokio::sync::mpsc;
use vintage_msg::{BlockChainMsg, BlockProduction, Tx};
use vintage_utils::SendMsg;

pub(super) async fn send_raw_tx_to_blockchain(sender: mpsc::Sender<BlockChainMsg>) {
    loop {
        let millis = thread_rng().gen_range(2000..=3000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(BlockChainMsg::RawTx(random_tx()));
    }
}

pub(super) async fn send_tx_to_blockchain(sender: mpsc::Sender<BlockChainMsg>) {
    loop {
        let millis = thread_rng().gen_range(500..=1000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(BlockChainMsg::Tx(random_tx()));
    }
}

pub(super) async fn send_block_to_blockchain(sender: mpsc::Sender<BlockChainMsg>) {
    let mut height_block = 1;

    loop {
        let ordered = thread_rng().gen_range(1..=100) <= 70; // 70%概率是顺序的

        let millis = thread_rng().gen_range(1000..=2000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(BlockChainMsg::BlockProduction(BlockProduction {
            block_height: if ordered {
                height_block
            } else {
                height_block + 1
            },
        }));

        let millis = thread_rng().gen_range(1000..=2000);
        tokio::time::sleep(Duration::from_millis(millis)).await;
        sender.send_msg(BlockChainMsg::BlockProduction(BlockProduction {
            block_height: if ordered {
                height_block + 1
            } else {
                height_block
            },
        }));

        height_block += 2;
    }
}

fn random_tx() -> Tx {
    let len = thread_rng().gen_range(100..=1000);
    let mut content = Vec::<u8>::with_capacity(len);
    for _ in 0..len {
        content.push(random())
    }

    Tx {
        id: Tx::new_id(),
        content,
    }
}
