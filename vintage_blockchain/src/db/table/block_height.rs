use crate::chain::GENESIS_BLOCK_HEIGHT;
use redb::{ReadableTable, StorageError};
use vintage_msg::BlockHeight;
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (BlockHeightTable, BlockHeightTableR, BlockHeightTableW) = ((), BlockHeight, "last_block_height")
}

impl<TABLE> BlockHeightTable<TABLE>
where
    TABLE: ReadableTable<(), BlockHeight>,
{
    pub fn get_block_height(&self) -> Result<BlockHeight, StorageError> {
        let height = match self.get(())? {
            Some(access) => access.value(),
            None => GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
