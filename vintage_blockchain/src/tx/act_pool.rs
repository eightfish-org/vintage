use crate::tx::ActId;
use std::collections::HashMap;
use vintage_msg::Act;

pub(crate) struct ActPool {
    map: HashMap<ActId, Act>,
}

impl ActPool {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn contains_act(&self, act_id: &ActId) -> bool {
        self.map.contains_key(act_id)
    }

    pub fn get_acts(&self, count: usize) -> (Vec<ActId>, Vec<Act>) {
        let mut act_ids = Vec::new();
        let mut acts = Vec::new();
        for (hash, act) in self.map.iter().take(count) {
            act_ids.push(hash.clone());
            acts.push(act.clone());
        }
        (act_ids, acts)
    }

    pub fn insert_act(&mut self, act_id: ActId, act: Act) {
        self.map.insert(act_id, act);
    }

    pub fn remove_acts(&mut self, act_ids: &[ActId]) {
        for id in act_ids {
            self.map.remove(id);
        }
    }
}
