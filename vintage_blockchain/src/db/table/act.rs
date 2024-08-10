use crate::tx::ActId;
use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::Act;
use vintage_utils::{define_redb_table, BincodeSerialize, RedbBytes, RedbBytesN};

define_redb_table! {
    pub(crate) (ActTable, ActTableR, ActTableW) = (RedbBytesN<32>, RedbBytes, "act")
}

impl<TABLE> ActTable<TABLE>
where
    TABLE: ReadableTable<RedbBytesN<32>, RedbBytes>,
{
    pub fn check_act_not_exists(&self, act_id: &ActId) -> anyhow::Result<()> {
        if self.exists(AsRef::<[u8; 32]>::as_ref(act_id))? {
            Err(anyhow!("act {} already exists id db", act_id))
        } else {
            Ok(())
        }
    }

    pub fn check_acts_not_exist(&self, ids: &[ActId]) -> anyhow::Result<()> {
        for id in ids {
            self.check_act_not_exists(id)?;
        }
        Ok(())
    }
}

impl<'db, 'txn> ActTableW<'db, 'txn> {
    pub fn insert_act(&mut self, act_id: &ActId, act: &Act) -> anyhow::Result<()> {
        let bytes = act.bincode_serialize()?;
        self.insert(AsRef::<[u8; 32]>::as_ref(act_id), &*bytes)?;
        Ok(())
    }
}
