use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use vintage_msg::{BlockHash, BlockHeight, BlockTimestamp, TxId};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize};

define_redb_table! {
    pub(crate) (Blocks, BlocksR, BlocksW) = (BlockHeight, Vec<u8>, "blocks")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlockInDb {
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
    pub tx_ids: Vec<TxId>,
}

impl<TABLE> Blocks<TABLE>
where
    TABLE: ReadableTable<BlockHeight, Vec<u8>>,
{
    pub fn get_block(&self, block_height: BlockHeight) -> anyhow::Result<Option<BlockInDb>>
    where
        TABLE: ReadableTable<BlockHeight, Vec<u8>>,
    {
        let option = self.get(block_height)?;
        match option {
            Some(access) => {
                let (block, _bytes_read) = BlockInDb::bincode_deserialize(&access.value())?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }
}

impl<'db, 'txn> BlocksW<'db, 'txn> {
    pub fn insert_block(
        &mut self,
        block_height: BlockHeight,
        block: &BlockInDb,
    ) -> anyhow::Result<()> {
        let bytes = block.bincode_serialize()?;
        self.insert(block_height, bytes)?;
        Ok(())
    }
}
