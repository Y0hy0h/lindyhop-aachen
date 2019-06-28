use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::Id;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required, because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Event {
    pub title: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required, because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
}

type Duration = u32;

impl Occurrence {
    pub fn end(&self) -> NaiveDateTime {
        use std::convert::TryInto;
        use std::ops::Add;
        self.start
            .add(chrono::Duration::minutes(self.duration.try_into().unwrap()))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)] // Hash, PartialEq, and Eq required, because Derive does not understand bounds on `Id`'s PhantomData. See https://github.com/rust-lang/rust/issues/26925
pub struct Location {
    pub name: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocationWithOccurrences {
    pub location: Location,
    pub occurrences: HashMap<Id<Occurrence>, Occurrence>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OccurrenceWithLocation {
    #[serde(flatten)]
    pub occurrence: Occurrence,
    pub location_id: Id<Location>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Overview {
    pub locations: HashMap<Id<Location>, Location>,
    pub events: HashMap<Id<Event>, EventWithOccurrences>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventWithOccurrences {
    pub event: Event,
    pub occurrences: Vec<OccurrenceWithLocation>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OccurrenceWithEvent {
    pub occurrence: OccurrenceWithLocation,
    pub event: Event,
}
