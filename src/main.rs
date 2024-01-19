#![allow(dead_code)]
use anyhow::Result;
use log::info;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const GENESIS_PREV_HASH: &str = "1984, George Orwell";
const LAST_BLOCK_POINTER: &str = "lbp";
const TABLE_BLOCKS: TableDefinition<&str, &str> = TableDefinition::new("blocks");
const TABLE_BLOCKS_FORTEST: TableDefinition<&str, &str> = TableDefinition::new("blocks_fortest");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct BlockHeader {
    hash: String,
    height: u64,
    prev_hash: String,
    timestamp: u64,
}

type BlockBody = Vec<String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    db: Db,
}

impl BlockChain {
    fn new() -> Self {
        Self::new_to_table(TABLE_BLOCKS)
    }

    fn new_to_table(table: TableDefinition<&str, &str>) -> Self {
        BlockChain {
            blocks: vec![],
            db: Db::open(table).expect("Db open failed, Fatal Error."),
        }
    }

    fn genesis() -> Block {
        let txs = vec!["The big brother is watching you.".to_string()];
        Block::new(0, GENESIS_PREV_HASH.to_string(), txs)
    }

    fn add_block(&mut self, block: Block) {
        self.persist_block(&block).expect("persistence error.");
        self.blocks.push(block);
    }

    fn persist_block(&mut self, block: &Block) -> Result<()> {
        self.persist_block_to_table(TABLE_BLOCKS, block)
    }

    fn persist_block_to_table(
        &mut self,
        table: TableDefinition<&str, &str>,
        block: &Block,
    ) -> Result<()> {
        let height = &block.header.height;
        let hash = &block.header.hash;
        let content = serde_json::to_string(&block)?;

        // store hash->block pair
        self.db.write_block_table(table, &hash, &content)?;
        // store height->hash pair
        self.db
            .write_block_table(table, &height.to_string(), &hash)?;
        // store the lbp->hash pair (last block pointer to hash)
        self.db
            .write_block_table(table, LAST_BLOCK_POINTER, &hash)?;

        Ok(())
    }

    fn retrieve_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        self.retrieve_block_by_hash_from_table(TABLE_BLOCKS, hash)
    }

    fn retrieve_block_by_hash_from_table(
        &self,
        table: TableDefinition<&str, &str>,
        hash: &str,
    ) -> Result<Option<Block>> {
        let content = self.db.read_block_table(table, hash)?;
        info!("{:?}", content);
        if let Some(content) = content {
            let b: Block = serde_json::from_str(&content)?;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    fn retrieve_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        self.retrieve_block_by_height_from_table(TABLE_BLOCKS, height)
    }

    fn retrieve_block_by_height_from_table(
        &self,
        table: TableDefinition<&str, &str>,
        height: u64,
    ) -> Result<Option<Block>> {
        let hash = self.db.read_block_table(table, &height.to_string())?;
        if let Some(hash) = hash {
            self.retrieve_block_by_hash(&hash)
        } else {
            Ok(None)
        }
    }

    fn populate_from_db(&mut self) -> Result<()> {
        self.populate_from_db_table(TABLE_BLOCKS)
    }

    fn populate_from_db_table(&mut self, table: TableDefinition<&str, &str>) -> Result<()> {
        // find last block hash from db
        let last_block_hash = self.db.read_block_table(table, LAST_BLOCK_POINTER)?;
        // let last_block_hash = last_block_hash.ok_or(anyhow!("no last_block_hash item in db."))?;
        if last_block_hash.is_none() {
            return Ok(());
        }
        let last_block_hash = last_block_hash.unwrap();

        // retrieve last block
        let block = self.retrieve_block_by_hash_from_table(table, &last_block_hash)?;
        // let block = block.ok_or(anyhow!("no block item in db."))?;
        if block.is_none() {
            return Ok(());
        }
        let block = block.unwrap();
        let mut prev_hash = block.header.prev_hash.clone();

        let mut blocks: Vec<Block> = vec![block];
        // iterate to old blockes by prev_hash
        while prev_hash != GENESIS_PREV_HASH {
            let block = self.retrieve_block_by_hash_from_table(table, &prev_hash)?;
            // let block = block.ok_or(anyhow!("no block item in db."))?;
            if block.is_none() {
                return Ok(());
            }
            let block = block.unwrap();
            prev_hash = block.header.prev_hash.clone();

            blocks.insert(0, block);
        }

        // contruct an instance of blockchain
        self.blocks = blocks;

        Ok(())
    }
}

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

#[derive(Debug)]
struct Db {
    db: Database,
}

impl Db {
    fn open(table: TableDefinition<&str, &str>) -> Result<Db> {
        let file = "vintage.db";
        let db = Database::create(file)?;

        // create table, if not exist
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(table)?;
            table.insert("t", "t")?;
        }
        write_txn.commit()?;

        Ok(Db { db })
    }

    fn write_block_table(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        content: &str,
    ) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(table)?;
            table.insert(key, content)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    fn read_block_table(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(table)?;
        let val = match table.get(key)? {
            Some(val) => val.value().to_owned(),
            None => return Ok(None),
        };

        Ok(Some(val))
    }

    fn drop_table(&self, table: TableDefinition<&str, &str>) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let table = write_txn.open_table(table)?;
            drop(table)
        }
        write_txn.commit()?;

        Ok(())
    }
}

#[test]
fn test_block_hash() {
    let block1 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    let block2 = Block::new(10, "aaabbbcccdddeeefff".to_string(), vec![]);
    assert_eq!(block1.header.height, block2.header.height);
    assert_eq!(block1.header.prev_hash, block2.header.prev_hash);
    // XXX: have little probabilities to fail
    assert_eq!(block1.header.timestamp, block2.header.timestamp);
    // XXX: have little probabilities to fail
    assert_eq!(block1.header.hash, block2.header.hash);

    assert_eq!(block1.body, block2.body);
}

#[test]
fn test_store_block_and_restore_block() {
    let mut blockchain = BlockChain::new_to_table(TABLE_BLOCKS_FORTEST);

    // initialization
    let genesis_block = BlockChain::genesis();
    let prev_hash = genesis_block.header.hash.clone();
    blockchain.add_block(genesis_block);

    let b1 = Block::new(1, prev_hash, vec![]);
    let prev_hash = b1.header.hash.clone();
    blockchain.add_block(b1);

    let b2 = Block::new(2, prev_hash, vec![]);
    blockchain.add_block(b2);

    let block_vec = blockchain.blocks.clone();

    blockchain
        .populate_from_db_table(TABLE_BLOCKS_FORTEST)
        .expect("error when populate from db");

    _ = blockchain.db.drop_table(TABLE_BLOCKS_FORTEST);

    for (i, block) in block_vec.into_iter().enumerate() {
        let block_tmp = blockchain.blocks[i].clone();
        assert_eq!(block, block_tmp);
    }
}
