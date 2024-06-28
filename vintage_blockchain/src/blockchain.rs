use crate::block::Block;
use crate::db::Db;
use log::info;
use redb::TableDefinition;

const GENESIS_PREV_HASH: &str = "1984, George Orwell";
const LAST_BLOCK_POINTER: &str = "lbp";
const TABLE_BLOCKS: TableDefinition<&str, &str> = TableDefinition::new("blocks");

#[derive(Debug)]
pub struct BlockChain {
    pub blocks: Vec<Block>,
    pub db: Db,
}

impl BlockChain {
    pub fn new() -> Self {
        Self::new_to_table(TABLE_BLOCKS)
    }

    pub fn new_to_table(table: TableDefinition<&str, &str>) -> Self {
        BlockChain {
            blocks: vec![],
            db: Db::open(table).expect("Db open failed, Fatal Error."),
        }
    }

    pub fn genesis() -> Block {
        let txs = vec!["The big brother is watching you.".to_string()];
        Block::new(0, GENESIS_PREV_HASH.to_string(), txs)
    }

    pub fn add_block(&mut self, block: Block) {
        self.add_block_to_table(TABLE_BLOCKS, block);
    }

    pub fn add_block_to_table(&mut self, table: TableDefinition<&str, &str>, block: Block) {
        self.persist_block_to_table(table, &block)
            .expect("persistence error.");
        self.blocks.push(block);
    }

    pub fn persist_block(&mut self, block: &Block) -> anyhow::Result<()> {
        self.persist_block_to_table(TABLE_BLOCKS, block)
    }

    fn persist_block_to_table(
        &mut self,
        table: TableDefinition<&str, &str>,
        block: &Block,
    ) -> anyhow::Result<()> {
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

    pub fn retrieve_block_by_hash(&self, hash: &str) -> anyhow::Result<Option<Block>> {
        self.retrieve_block_by_hash_from_table(TABLE_BLOCKS, hash)
    }

    fn retrieve_block_by_hash_from_table(
        &self,
        table: TableDefinition<&str, &str>,
        hash: &str,
    ) -> anyhow::Result<Option<Block>> {
        let content = self.db.read_block_table(table, hash)?;
        info!("{:?}", content);
        if let Some(content) = content {
            let b: Block = serde_json::from_str(&content)?;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    pub fn retrieve_block_by_height(&self, height: u64) -> anyhow::Result<Option<Block>> {
        self.retrieve_block_by_height_from_table(TABLE_BLOCKS, height)
    }

    fn retrieve_block_by_height_from_table(
        &self,
        table: TableDefinition<&str, &str>,
        height: u64,
    ) -> anyhow::Result<Option<Block>> {
        let hash = self.db.read_block_table(table, &height.to_string())?;
        if let Some(hash) = hash {
            self.retrieve_block_by_hash(&hash)
        } else {
            Ok(None)
        }
    }

    pub fn populate_from_db(&mut self) -> anyhow::Result<()> {
        self.populate_from_db_table(TABLE_BLOCKS)
    }

    pub fn populate_from_db_table(
        &mut self,
        table: TableDefinition<&str, &str>,
    ) -> anyhow::Result<()> {
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
