use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

pub trait WithId {
    type Id;
    fn id(&self) -> &Self::Id;
}

pub struct Pool<ID, VALUE> {
    map: HashMap<ID, VALUE>,
    capacity: usize,
}

impl<ID, VALUE> Pool<ID, VALUE>
where
    ID: Hash + Eq + Clone,
    VALUE: WithId<Id = ID>,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn contains_id<Q: ?Sized>(&self, id: &Q) -> bool
    where
        Q: Hash + Eq,
        ID: Borrow<Q>,
    {
        self.map.contains_key(id)
    }

    pub fn contains<Q: ?Sized>(&self, value: &VALUE) -> bool
    where
        Q: Hash + Eq,
        ID: Borrow<Q>,
    {
        self.map.contains_key(value.id().borrow())
    }

    pub fn insert(&mut self, value: VALUE)
    where
        VALUE: WithId<Id = ID>,
    {
        self.map.insert(value.id().clone(), value);
    }

    pub fn take_map(&mut self) -> HashMap<ID, VALUE> {
        mem::replace(&mut self.map, HashMap::with_capacity(self.capacity))
    }
}
