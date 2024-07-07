mod read_transaction;
mod table;
mod write_transaction;

pub(crate) use self::read_transaction::*;
pub(crate) use self::table::*;
pub(crate) use self::write_transaction::*;

pub(crate) struct Db {
    database: redb::Database,
}

impl Db {
    pub fn new(database: redb::Database) -> Self {
        Self { database }
    }

    pub fn begin_read(&self) -> Result<DbRead, redb::TransactionError> {
        let transaction = self.database.begin_read()?;
        Ok(DbRead::new(transaction))
    }

    pub fn begin_write(&self) -> Result<DbWrite, redb::TransactionError> {
        let transaction = self.database.begin_write()?;
        Ok(DbWrite::new(transaction))
    }
}
