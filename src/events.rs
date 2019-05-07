use std::collections::{BTreeMap, HashMap};

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use super::id_map::{Id, IdMap, UnsafeId};

// Types

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
    pub occurrences: Vec<Occurrence>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub location_id: UnsafeId,
}

type Duration = u64;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    pub name: String,
    pub address: String,
}

pub type Locations = IdMapCru<Location>;

pub type Events = IdMap<Event>;

pub type Occurrences<'a> = Vec<(&'a Occurrence, &'a Event)>;

#[derive(Serialize, Clone, Debug)]
pub struct Store {
    pub locations: Locations,
    pub events: Events,
}

impl Store {
    pub fn from(locations: Locations, events: Events) -> Store {
        Store {
            locations: locations,
            events: events,
        }
    }

    pub fn occurrences(&self) -> Occurrences {
        let mut occurrences = self
            .events
            .iter()
            .flat_map(|(_, event)| {
                event
                    .occurrences
                    .iter()
                    .map(move |occurrence| (occurrence, event))
            })
            .collect::<Occurrences>();
        occurrences.sort_by(|(first, _), (second, _)| first.start.cmp(&second.start));
        occurrences
    }

    pub fn occurrences_by_date(&self) -> BTreeMap<NaiveDate, Occurrences> {
        self.occurrences()
            .into_iter()
            .fold(BTreeMap::new(), |mut acc, entry| {
                acc.entry(entry.0.start.date())
                    .and_modify(|entries| entries.push(entry))
                    .or_insert(vec![entry]);
                acc
            })
    }

    pub fn delete_location(&mut self, id: &Id<Location>) -> Result<Location, Vec<Id<Event>>> {
        let dependent_events: Vec<Id<Event>> = self
            .events
            .iter()
            .filter_map(|(event_id, event)| {
                let refers_to_location = event
                    .occurrences
                    .iter()
                    .any(|occurrence| occurrence.location_id == id.to_unsafe());

                if refers_to_location {
                    Some(event_id)
                } else {
                    None
                }
            })
            .collect();

        if dependent_events.len() > 0 {
            Err(dependent_events)
        } else {
            Ok(self.locations.0.remove(&id))
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct IdMapCru<I>(IdMap<I>);

impl<I> IdMapCru<I> {
    pub fn new() -> IdMapCru<I> {
        IdMapCru(IdMap::new())
    }
    pub fn init(raw: HashMap<UnsafeId, I>) -> IdMap<I> {
        IdMap::init(raw)
    }

    pub fn insert(&mut self, item: I) -> Id<I> {
        self.0.insert(item)
    }

    pub fn values(&self) -> impl Iterator<Item = &I> {
        self.0.values()
    }

    pub fn get(&self, id: &Id<I>) -> &I {
        self.0.get(id)
    }

    pub fn set(&mut self, id: Id<I>, new_item: I) {
        self.0.set(id, new_item);
    }

    pub fn validate(&self, unsafe_id: UnsafeId) -> Option<(Id<I>)> {
        self.0.validate(unsafe_id)
    }
}
