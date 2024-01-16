#![allow(dead_code)]
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct BlockHeader {
    hash: String,
    height: u64,
    prev_hash: String,
    timestamp: u64,
}

type BlockBody = Vec<String>;

#[derive(Debug, Clone)]
struct Block {
    header: BlockHeader,
    body: BlockBody,
}

impl Block {
    fn new(height: u64, prev_hash: String, body: BlockBody) -> Self {
        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp() as u64;
        let mut bh = BlockHeader {
            hash: String::new(),
            height,
            prev_hash,
            timestamp,
        };
        bh.hash = Self::calc_block_hash(height, &bh.prev_hash, timestamp, &body);
        Block { header: bh, body }
    }

    fn calc_block_hash(height: u64, prev_hash: &str, timestamp: u64, body: &Vec<String>) -> String {
        let concated_str = vec![
            height.to_string(),
            prev_hash.to_string(),
            timestamp.to_string(),
            body.concat(),
        ]
        .concat();

        let mut hasher = Sha256::new();
        hasher.update(concated_str.as_bytes());
        hex::encode(hasher.finalize().as_slice())
    }
}

#[derive(Debug)]
struct BlockChain {
    blocks: Vec<Block>,
}

impl BlockChain {
    fn new() -> Self {
        BlockChain { blocks: vec![] }
    }

    fn genesis() -> Block {
        let txs = vec!["The big brother is watching you.".to_string()];
        Block::new(0, "1984, George Orwell".to_string(), txs)
    }

    fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

fn main() {
    let mut blockchain = BlockChain::new();
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

    println!("{:#?}", blockchain);
}

#[test]
fn test_block_hash() {
    let block1 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    let block2 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    assert_eq!(block1.header.height, block2.header.height);
    assert_eq!(block1.header.prev_hash, block2.header.prev_hash);
    // XXX: have little probability to fail
    assert_eq!(block1.header.timestamp, block2.header.timestamp);
    // XXX: have little probability to fail
    assert_eq!(block1.header.hash, block2.header.hash);

    assert_eq!(block1.body, block2.body);
}
