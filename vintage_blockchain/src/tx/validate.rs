use crate::table_txs;
use anyhow::anyhow;
use redb::{Database, ReadableTable};
use vintage_msg::Tx;
use vintage_utils::Validate;

pub(crate) struct TxValidate<'a> {
    pub db: &'a Database,
}

impl<'a> TxValidate<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }
}

impl<'a> Validate for TxValidate<'a> {
    type Data = Tx;
    type Error = anyhow::Error;

    fn validate(&self, data: &Self::Data) -> Result<(), Self::Error> {
        let transaction = self.db.begin_read()?;
        let table = table_txs::open_readonly_table(&transaction)?;
        let result = match table.get(data.id)? {
            Some(_) => Err(anyhow!("Tx already exists")),
            None => Ok(()),
        };
        result
    }
}
