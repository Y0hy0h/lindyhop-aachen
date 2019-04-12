use std::collections::HashMap;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use super::id_map::{Id, IdMap, UnsafeId};

// Types

#[derive(Serialize, Clone)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
    pub occurrences: Vec<Occurrence>,
}

#[derive(Serialize, Clone)]
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub location_id: Id<Location>,
}

type Duration = u64;

#[derive(Serialize, Deserialize, Clone)]
pub struct Location {
    pub name: String,
    pub address: String,
}

pub type Locations = IdMap<Location>;

pub type Events = IdMap<Event>;

#[derive(Serialize, Clone)]
pub struct Store {
    pub locations: Locations,
    pub events: Events,
}

impl Store {
    fn resolve(locations: Locations, ref_events: HashMap<UnsafeId, RefEvent>) -> Option<Store> {
        ref_events
            .into_iter()
            .map(|(key, ref_event)| ref_event.resolve(&locations).map(|event| (key, event)))
            .collect::<Option<HashMap<UnsafeId, Event>>>()
            .map(|events| Store {
                locations: locations,
                events: IdMap::init(events),
            })
    }
}

#[derive(Deserialize)]
pub struct RefStore {
    pub locations: Locations,
    pub ref_events: HashMap<UnsafeId, RefEvent>,
}

#[derive(Deserialize)]
pub struct RefEvent {
    pub name: String,
    pub teaser: String,
    pub description: String,
    pub occurrences: Vec<RefOccurrence>,
}

impl RefEvent {
    pub fn resolve(self, locations: &Locations) -> Option<Event> {
        self.occurrences
            .iter()
            .map(|occurrence| occurrence.resolve(locations))
            .collect::<Option<Vec<Occurrence>>>()
            .map(|occurrences| Event {
                name: self.name,
                teaser: self.teaser,
                description: self.description,
                occurrences,
            })
    }
}

#[derive(Deserialize)]
pub struct RefOccurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
    #[serde(rename = "location_id")]
    pub unsafe_location_id: UnsafeId,
}

impl RefOccurrence {
    pub fn resolve(&self, locations: &Locations) -> Option<Occurrence> {
        locations
            .validate(self.unsafe_location_id)
            .map(|location_id| Occurrence {
                start: self.start,
                duration: self.duration,
                location_id,
            })
    }
}
