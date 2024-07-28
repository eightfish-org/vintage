use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{BlockHeight, StateRoot};
use vintage_utils::{define_redb_table, RedbBytesN};

define_redb_table! {
    pub(crate) (StateRootTable, StateRootTableR, StateRootTableW) = (BlockHeight, RedbBytesN<32>, "state_root")
}

impl<TABLE> StateRootTable<TABLE>
where
    TABLE: ReadableTable<BlockHeight, RedbBytesN<32>>,
{
    pub fn get_state_root(&self, block_height: BlockHeight) -> anyhow::Result<StateRoot> {
        match self.get(&block_height)? {
            Some(access) => Ok(access.value().to_owned()),
            None => Err(anyhow!("state root {} not found", block_height)),
        }
    }
}
