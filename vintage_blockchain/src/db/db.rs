use crate::db::{BlockChainDbRead, BlockChainDbWrite, BlockInDb};
use redb::{Database, TransactionError};
use std::path::Path;
use vintage_msg::{Block, BlockHash, BlockHeight, TxId};

pub(crate) struct BlockChainDb {
    database: Database,
}

impl BlockChainDb {
    pub fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let db = Self {
            database: Database::create(path)?,
        };
        db.create_tables()?;
        Ok(db)
    }

    fn begin_read(&self) -> Result<BlockChainDbRead, TransactionError> {
        let transaction = self.database.begin_read()?;
        Ok(BlockChainDbRead::new(transaction))
    }

    fn begin_write(&self) -> Result<BlockChainDbWrite, TransactionError> {
        let transaction = self.database.begin_write()?;
        Ok(BlockChainDbWrite::new(transaction))
    }
}

impl BlockChainDb {
    pub fn check_tx_not_exists(&self, id: TxId) -> anyhow::Result<()> {
        self.begin_read()?.open_txs()?.check_tx_not_exists(id)
    }

    pub fn check_txs_not_exist(&self, ids: &[TxId]) -> anyhow::Result<()> {
        self.begin_read()?.open_txs()?.check_txs_not_exist(ids)
    }

    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        Ok(self
            .begin_read()?
            .open_last_block_height()?
            .get_last_block_height()?)
    }

    pub fn get_block_hash(&self, block_height: BlockHeight) -> anyhow::Result<BlockHash> {
        Ok(self
            .begin_read()?
            .open_blocks()?
            .get_block(block_height)?
            .hash)
    }
}

impl BlockChainDb {
    fn create_tables(&self) -> anyhow::Result<()> {
        let db_write = self.begin_write()?;

        db_write.open_last_block_height()?;
        db_write.open_blocks()?;
        db_write.open_txs()?;

        db_write.commit()?;
        Ok(())
    }

    // complete all operations within a single transaction
    pub fn write_block(&self, block: &Block) -> anyhow::Result<()> {
        let db_write = self.begin_write()?;

        // update last_block_height
        {
            let mut table_lbh = db_write.open_last_block_height()?;
            table_lbh.insert((), block.header.height)?;
        }

        let mut tx_ids = Vec::<TxId>::new();
        // insert txs
        {
            let mut table_txs = db_write.open_txs()?;
            for tx in &block.body.txs {
                tx_ids.push(tx.id);
                table_txs.insert(tx.id, &tx.content)?;
            }
        }

        // insert block
        {
            let mut table_blocks = db_write.open_blocks()?;
            table_blocks.insert_block(
                block.header.height,
                &BlockInDb {
                    hash: block.header.hash.clone(),
                    timestamp: block.header.timestamp,
                    tx_ids,
                },
            )?;
        }

        db_write.commit()?;
        Ok(())
    }
}
