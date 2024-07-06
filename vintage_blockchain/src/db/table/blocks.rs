use crate::db::DbTable;
use redb::{ReadableTable, Table};
use serde::{Deserialize, Serialize};
use vintage_msg::{BlockHash, BlockHeight, BlockTimestamp, TxId};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize};

define_redb_table!(BlockHeight, Vec<u8>, "last_block_height");

#[derive(Serialize, Deserialize)]
pub(crate) struct BlockInDb {
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
    pub tx_ids: Vec<TxId>,
}

impl DbTable {
    pub fn get_block<TABLE>(
        table: &TABLE,
        block_height: BlockHeight,
    ) -> anyhow::Result<Option<BlockInDb>>
    where
        TABLE: ReadableTable<BlockHeight, Vec<u8>>,
    {
        let option = table.get(block_height)?;
        match option {
            Some(access) => {
                let (block, _bytes_read) = BlockInDb::bincode_deserialize(&access.value())?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    pub fn insert_block(
        table: &mut Table<BlockHeight, Vec<u8>>,
        block_height: BlockHeight,
        block: &BlockInDb,
    ) -> anyhow::Result<()> {
        let bytes = block.bincode_serialize()?;
        table.insert(block_height, bytes)?;
        Ok(())
    }
}
