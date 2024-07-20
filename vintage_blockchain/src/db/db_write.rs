use crate::db::Blocks;
use crate::db::Txs;
use crate::db::{BlocksW, LastBlockHeight, LastBlockHeightW, TxsW};
use redb::{CommitError, TableError, WriteTransaction};

pub(crate) struct BlockChainDbWrite<'db> {
    transaction: WriteTransaction<'db>,
}

impl<'db> BlockChainDbWrite<'db> {
    pub fn new(transaction: WriteTransaction<'db>) -> Self {
        Self { transaction }
    }

    pub fn commit(self) -> Result<(), CommitError> {
        self.transaction.commit()
    }

    pub fn open_last_block_height<'txn>(
        &'txn self,
    ) -> Result<LastBlockHeightW<'db, 'txn>, TableError> {
        LastBlockHeight::open_writable_table(&self.transaction)
    }

    pub fn open_blocks<'txn>(&'txn self) -> Result<BlocksW<'db, 'txn>, TableError> {
        Blocks::open_writable_table(&self.transaction)
    }

    pub fn open_txs<'txn>(&'txn self) -> Result<TxsW<'db, 'txn>, TableError> {
        Txs::open_writable_table(&self.transaction)
    }
}
