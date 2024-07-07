use crate::genesis::GENESIS_BLOCK_HEIGHT;
use redb::ReadableTable;
use vintage_msg::BlockHeight;
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (LastBlockHeight, LastBlockHeightW) = ((), BlockHeight, "last_block_height")
}

impl<TABLE> LastBlockHeight<TABLE>
where
    TABLE: ReadableTable<(), BlockHeight>,
{
    pub fn get_last_block_height(&self) -> anyhow::Result<BlockHeight> {
        let height = match self.get(())? {
            Some(value) => value.value(),
            None => GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
