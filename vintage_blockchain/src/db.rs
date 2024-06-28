use log::info;
use redb::{Database, ReadableTable, TableDefinition};

#[derive(Debug)]
pub struct Db {
    db: Database,
}

impl Db {
    pub fn open(table: TableDefinition<&str, &str>) -> anyhow::Result<Db> {
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

    pub fn write_block_table(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
        content: &str,
    ) -> anyhow::Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(table)?;
            table.insert(key, content)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    pub fn read_block_table(
        &self,
        table: TableDefinition<&str, &str>,
        key: &str,
    ) -> anyhow::Result<Option<String>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(table)?;
        let val = match table.get(key)? {
            Some(val) => val.value().to_owned(),
            None => return Ok(None),
        };

        Ok(Some(val))
    }

    pub fn drop_table(&self, table: TableDefinition<&str, &str>) -> anyhow::Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let result = write_txn.delete_table(table)?;
            info!("{:?}", result);
        }
        write_txn.commit()?;

        Ok(())
    }
}
