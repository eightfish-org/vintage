use redb::ReadableTable;
use vintage_msg::BlockHeight;
use vintage_utils::define_redb_table;

const GENESIS_BLOCK_HEIGHT: BlockHeight = 0;

define_redb_table!((), BlockHeight, "last_block_height");

pub(crate) fn db_get_last_block_height<TABLE>(table: &TABLE) -> anyhow::Result<BlockHeight>
where
    TABLE: ReadableTable<(), BlockHeight>,
{
    let height = match table.get(())? {
        Some(value) => value.value(),
        None => GENESIS_BLOCK_HEIGHT,
    };
    Ok(height)
}
