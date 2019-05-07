use std::collections::HashMap;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdMap<I>(HashMap<UnsafeId, I>);

impl<I> IdMap<I> {
    pub fn new() -> IdMap<I> {
        IdMap(HashMap::new())
    }

    pub fn init(raw: HashMap<UnsafeId, I>) -> IdMap<I> {
        IdMap(raw)
    }

    pub fn insert(&mut self, item: I) -> Id<I> {
        let uuid = Uuid::new_v4();
        self.0.insert(uuid, item);

        Id::init(uuid)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Id<I>, &I)> {
        self.0.iter().map(|(k, v)| (Id::init(*k), v))
    }

    pub fn values(&self) -> impl Iterator<Item = &I> {
        self.0.values()
    }

    pub fn get(&self, id: &Id<I>) -> &I {
        self.0.get(&id.raw).expect("The id was invalid.")
    }

    pub fn set(&mut self, id: Id<I>, new_item: I) {
        self.0.insert(id.raw, new_item);
    }

    pub fn remove(&mut self, id: &Id<I>) -> I {
        self.0.remove(&id.raw).expect("The id was invalid.")
    }

    pub fn validate(&self, unsafe_id: UnsafeId) -> Option<(Id<I>)> {
        if self.0.contains_key(&unsafe_id) {
            Some(Id::init(unsafe_id))
        } else {
            None
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(transparent)]
pub struct Id<T> {
    raw: Uuid,
    #[serde(skip)]
    phantom: PhantomData<T>,
}

pub type UnsafeId = Uuid;

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Id<T>) -> bool {
        self.raw == other.raw
    }
}

impl<T> Eq for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id {
            raw: self.raw.clone(),
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for Id<T> {}

impl<T> std::hash::Hash for Id<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.raw.hash(state)
    }
}

impl<T> Id<T> {
    fn init(raw: Uuid) -> Id<T> {
        Id {
            raw,
            phantom: PhantomData,
        }
    }

    pub fn to_unsafe(&self) -> UnsafeId {
        self.raw.clone()
    }
}
