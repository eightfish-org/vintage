mod read_transaction;
mod table;
mod write_transaction;

pub(crate) use self::read_transaction::*;
pub(crate) use self::table::*;
pub(crate) use self::write_transaction::*;
use std::path::Path;

use redb::{Database, TransactionError};

pub(crate) struct Db {
    database: Database,
}

impl Db {
    pub fn create(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let database = Database::create(path)?;
        let db = Self { database };
        {
            let db_write = db.begin_write()?;
            db_write.open_last_block_height()?;
            db_write.open_blocks()?;
            db_write.open_txs()?;
            db_write.commit()?;
        }
        Ok(db)
    }

    pub fn begin_read(&self) -> Result<DbRead, TransactionError> {
        let transaction = self.database.begin_read()?;
        Ok(DbRead::new(transaction))
    }

    pub fn begin_write(&self) -> Result<DbWrite, TransactionError> {
        let transaction = self.database.begin_write()?;
        Ok(DbWrite::new(transaction))
    }
}
