use anyhow::anyhow;
use redb::ReadableTable;
use vintage_msg::{ActId, EntityState};
use vintage_utils::{define_redb_table, BincodeDeserialize, BincodeSerialize, RedbBytes};

define_redb_table! {
    pub(crate) (ActEntitiesStateTable, ActEntitiesStateTableR, ActEntitiesStateTableW) = (ActId, RedbBytes, "act_entities_state")
}

impl<TABLE> ActEntitiesStateTable<TABLE>
where
    TABLE: ReadableTable<ActId, RedbBytes>,
{
    pub fn check_act_not_exists(&self, id: ActId) -> anyhow::Result<()> {
        if self.exists(id)? {
            Err(anyhow!("act state {} already exists id db", id))
        } else {
            Ok(())
        }
    }

    pub fn get_act_state(&self, act_id: ActId) -> anyhow::Result<Vec<EntityState>> {
        match self.get(act_id)? {
            Some(access) => {
                let (block, _bytes_read) =
                    Vec::<EntityState>::bincode_deserialize(&access.value())?;
                Ok(block)
            }
            None => Err(anyhow!("act state {} not found", act_id)),
        }
    }

    pub fn get_acts_state(&self, ids: &[ActId]) -> anyhow::Result<Vec<EntityState>> {
        let mut vec = Vec::new();
        for id in ids {
            let vec_2 = self.get_act_state(id.clone())?;
            vec.extend(vec_2)
        }
        Ok(vec)
    }
}

impl<'db, 'txn> ActEntitiesStateTableW<'db, 'txn> {
    pub fn insert_act_state(
        &mut self,
        act_id: ActId,
        entities_state: &Vec<EntityState>,
    ) -> anyhow::Result<()> {
        let bytes = entities_state.bincode_serialize()?;
        self.insert(act_id, &*bytes)?;
        Ok(())
    }
}
