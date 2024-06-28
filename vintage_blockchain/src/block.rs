use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    pub hash: String,
    pub height: u64,
    pub prev_hash: String,
    pub timestamp: u64,
}

type BlockBody = Vec<String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl Block {
    pub fn new(height: u64, prev_hash: String, body: BlockBody) -> Self {
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
