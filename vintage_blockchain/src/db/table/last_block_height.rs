use crate::block::genesis::GENESIS_BLOCK_HEIGHT;
use redb::{ReadableTable, StorageError};
use vintage_msg::BlockHeight;
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
            Some(value) => value.value(),
            None => GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
