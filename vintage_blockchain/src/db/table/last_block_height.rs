use crate::db::DbTable;
use crate::genesis::GENESIS_BLOCK_HEIGHT;
use redb::ReadableTable;
use vintage_msg::BlockHeight;
use vintage_utils::define_redb_table;

define_redb_table!((), BlockHeight, "last_block_height");

impl DbTable {
    pub fn get_last_block_height<TABLE>(table: &TABLE) -> anyhow::Result<BlockHeight>
    where
        TABLE: ReadableTable<(), BlockHeight>,
    {
        let height = match table.get(())? {
            Some(value) => value.value(),
            None => GENESIS_BLOCK_HEIGHT,
        };
        Ok(height)
    }
}
