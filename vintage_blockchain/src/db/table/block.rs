use crate::chain::BlockState;
use crate::tx::TxId;
use anyhow::anyhow;
use redb::ReadableTable;
use serde::{Deserialize, Serialize};
use vintage_msg::{BlockHash, BlockHeight, BlockTimestamp, WasmId};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (BlockTable, BlockTableR, BlockTableW) = (BlockHeight, RedbBytes, "block")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BlockInDb {
    pub hash: BlockHash,
    pub timestamp: BlockTimestamp,
    pub state: BlockState,
    pub act_tx_ids: Vec<TxId>,
    pub ue_tx_ids: Vec<TxId>,
    pub wasm_ids: Vec<WasmId>,
}

impl<TABLE> BlockTable<TABLE>
where
    TABLE: ReadableTable<BlockHeight, RedbBytes>,
{
    pub fn get_block(&self, height: BlockHeight) -> anyhow::Result<BlockInDb> {
        match self.get(height)? {
            Some(access) => {
                let (value, _bytes_read) = BlockInDb::bincode_deserialize(access.value())?;
                Ok(value)
            }
            None => Err(anyhow!("block {} not found", height)),
        }
    }
}

impl<'db, 'txn> BlockTableW<'db, 'txn> {
    pub fn insert_block(&mut self, height: BlockHeight, block: &BlockInDb) -> anyhow::Result<()> {
        let bytes = block.bincode_serialize()?;
        self.insert(height, bytes.as_slice())?;
        Ok(())
    }
}
