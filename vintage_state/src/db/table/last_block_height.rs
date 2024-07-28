use redb::{ReadableTable, StorageError};
use vintage_msg::{BlockHeight, GENESIS_BLOCK_HEIGHT};
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (LastBlockHeightTable, LastBlockHeightTableR, LastBlockHeightTableW) = ((), BlockHeight, "last_block_height")
}

impl<TABLE> LastBlockHeightTable<TABLE>
where
    TABLE: ReadableTable<(), BlockHeight>,
{
    pub fn get_last_block_height(&self) -> Result<BlockHeight, StorageError> {
        let height = match self.get(())? {
            Some(access) => access.value(),
            None => GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
