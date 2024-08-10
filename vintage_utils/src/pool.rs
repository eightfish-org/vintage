use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

pub trait WithId {
    type Id;
    fn id(&self) -> &Self::Id;
}

pub struct Pool<VALUE>
where
    VALUE: WithId,
{
    map: HashMap<VALUE::Id, VALUE>,
}

impl<VALUE> Pool<VALUE>
where
    VALUE: WithId + Clone,
    VALUE::Id: Hash + Eq + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn contains_id<Q>(&self, id: &Q) -> bool
    where
        Q: ?Sized + Hash + Eq,
        VALUE::Id: Borrow<Q>,
    {
        self.map.contains_key(id)
    }

    pub fn get_values(&self, count: usize) -> Vec<VALUE> {
        self.map
            .values()
            .take(count)
            .cloned()
            .collect::<Vec<VALUE>>()
    }

    pub fn insert(&mut self, value: VALUE) {
        self.map.insert(value.id().clone(), value);
    }

    pub fn remove<Q>(&mut self, id: &Q) -> Option<VALUE>
    where
        Q: ?Sized + Hash + Eq,
        VALUE::Id: Borrow<Q>,
    {
        self.map.remove(id)
    }

    pub fn remove_by_ids(&mut self, ids: &[VALUE::Id]) {
        for id in ids {
            self.map.remove(id);
        }
    }
}
