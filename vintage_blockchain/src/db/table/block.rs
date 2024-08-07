use anyhow::anyhow;
use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use vintage_msg::{ActId, BlockHash, BlockHeight, BlockTimestamp};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (BlockTable, BlockTableR, BlockTableW) = (BlockHeight, RedbBytes, "block")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlockInDb {
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
    pub act_ids: Vec<ActId>,
}

impl<TABLE> BlockTable<TABLE>
where
    TABLE: ReadableTable<BlockHeight, RedbBytes>,
{
    pub fn get_block(&self, block_height: BlockHeight) -> anyhow::Result<BlockInDb> {
        match self.get(block_height)? {
            Some(access) => {
                let (block, _bytes_read) = BlockInDb::bincode_deserialize(&access.value())?;
                Ok(block)
            }
            None => Err(anyhow!("block {} not found", block_height)),
        }
    }
}

impl<'db, 'txn> BlockTableW<'db, 'txn> {
    pub fn insert_block(
        &mut self,
        block_height: BlockHeight,
        block: &BlockInDb,
    ) -> anyhow::Result<()> {
        let bytes = block.bincode_serialize()?;
        self.insert(block_height, &*bytes)?;
        Ok(())
    }
}
