use redb::Database;
use vintage_msg::Block;
use vintage_utils::Validate;

pub(crate) struct BlockValidate<'a> {
    database: &'a Database,
}

impl<'a> BlockValidate<'a> {
    pub fn new(database: &'a Database) -> Self {
        Self { database }
    }
}

impl<'a> Validate for BlockValidate<'a> {
    type Data = Block;
    type Error = anyhow::Error;

    #[allow(unused_variables)]
    fn validate(&self, data: &Self::Data) -> Result<(), Self::Error> {
        Ok(())
    }
}
