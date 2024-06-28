use vintage_blockchain::{Block, BlockChain};

fn main() {
    env_logger::init();

    let mut blockchain = BlockChain::new();
    blockchain
        .populate_from_db()
        .expect("error when populate from db");

    if blockchain.blocks.is_empty() {
        // initialization
        let genesis_block = BlockChain::genesis();
        let prev_hash = genesis_block.header.hash.clone();
        blockchain.add_block(genesis_block);

        let b1 = Block::new(1, prev_hash, vec![]);
        let prev_hash = b1.header.hash.clone();
        blockchain.add_block(b1);

        let b2 = Block::new(2, prev_hash, vec![]);
        let prev_hash = b2.header.hash.clone();
        blockchain.add_block(b2);

        let b3 = Block::new(3, prev_hash, vec![]);
        let prev_hash = b3.header.hash.clone();
        blockchain.add_block(b3);

        let b4 = Block::new(4, prev_hash, vec![]);
        let prev_hash = b4.header.hash.clone();
        blockchain.add_block(b4);

        let b5 = Block::new(5, prev_hash, vec![]);
        // let prev_hash = b5.header.hash.clone();
        blockchain.add_block(b5);
    } else {
        println!("has restored from db.");
        // do something else
    }

    println!("{:#?}", blockchain);
}
