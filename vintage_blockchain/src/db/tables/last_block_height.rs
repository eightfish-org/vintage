use crate::db::DB;
use redb::ReadableTable;
use vintage_msg::BlockHeight;
use vintage_utils::define_redb_table;

define_redb_table!((), BlockHeight, "last_block_height");

impl DB {
    const GENESIS_BLOCK_HEIGHT: BlockHeight = 0;

    pub fn get_last_block_height<TABLE>(table: &TABLE) -> anyhow::Result<BlockHeight>
    where
        TABLE: ReadableTable<(), BlockHeight>,
    {
        let height = match table.get(())? {
            Some(value) => value.value(),
            None => Self::GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
