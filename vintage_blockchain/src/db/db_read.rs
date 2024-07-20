use crate::db::Blocks;
use crate::db::Txs;
use crate::db::{BlocksR, LastBlockHeight, LastBlockHeightR, TxsR};
use redb::{ReadTransaction, TableError};

pub(crate) struct BlockChainDbRead<'db> {
    transaction: ReadTransaction<'db>,
}

impl<'db> BlockChainDbRead<'db> {
    pub fn new(transaction: ReadTransaction<'db>) -> Self {
        Self { transaction }
    }

    pub fn open_last_block_height(&self) -> Result<LastBlockHeightR, TableError> {
        LastBlockHeight::open_table(&self.transaction)
    }

    pub fn open_blocks(&self) -> Result<BlocksR, TableError> {
        Blocks::open_table(&self.transaction)
    }

    pub fn open_txs(&self) -> Result<TxsR, TableError> {
        Txs::open_table(&self.transaction)
    }
}
