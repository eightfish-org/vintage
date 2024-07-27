use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{ActContent, ActId};
use vintage_utils::define_redb_table;

define_redb_table! {
    pub(crate) (ActTable, ActTableR, ActTableW) = (ActId, ActContent, "act")
}

impl<TABLE> ActTable<TABLE>
where
    TABLE: ReadableTable<ActId, ActContent>,
{
    pub fn check_act_not_exists(&self, id: ActId) -> anyhow::Result<()> {
        if self.exists(id)? {
            Err(anyhow!("act {} already exists id db", id))
        } else {
            Ok(())
        }
    }

    pub fn check_acts_not_exist(&self, ids: &[ActId]) -> anyhow::Result<()> {
        for id in ids {
            self.check_act_not_exists(id.clone())?;
        }
        Ok(())
    }
}
