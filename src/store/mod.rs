mod db;
#[macro_use]
pub mod action;
pub mod routes;

use std::collections::HashMap;
use std::iter::FromIterator;

use chrono::NaiveDateTime;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::{fairing, fairing::Fairing, Rocket};
use uuid::Uuid;

use db::{SqlEvent, SqlLocation, SqlOccurrence};
use diesel::result::QueryResult;
use diesel::{self, prelude::*};
use serde::{Deserialize, Serialize};

use action::Actions;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Occurrence {
    pub start: NaiveDateTime,
    pub duration: Duration,
    pub event_id: Id,
    pub location_id: Id,
}

type Duration = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct Location {
    pub name: String,
    pub address: String,
}

mod location_action {
    use super::db::schema::locations::{dsl::locations as schema, table};
    use super::*;

    derive_actions!(Location, SqlLocation);
}

mod event_actions {
    use super::db::schema::events::{dsl::events as schema, table};
    use super::*;

    derive_actions!(Event, SqlEvent);
}

mod occurrence_actions {
    use super::db::schema::occurrences::{dsl::occurrences as schema, table};
    use super::*;

    derive_actions!(Occurrence, SqlOccurrence);
}

#[derive(Deserialize, Serialize)]
pub struct Overview {
    pub locations: HashMap<Id, Location>,
    pub events: HashMap<Id, (Event, Vec<Occurrence>)>,
}

pub type Id = Uuid;

pub struct Store(db::Connection);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> Overview {
        let locs: HashMap<Id, Location> = self.all();
        let evts: HashMap<Id, (Event, Vec<Occurrence>)> = self.all();

        Overview {
            locations: locs,
            events: evts,
        }
    }
}

impl Actions<(Event, Vec<Occurrence>)> for Store {
    type Id = Id;

    fn all(&self) -> HashMap<Self::Id, (Event, Vec<Occurrence>)> {
        use db::schema::events::dsl::events;
        use diesel::BelongingToDsl;

        events
            .load::<SqlEvent>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_event| {
                let occrs: Vec<Occurrence> = SqlOccurrence::belonging_to(&sql_event)
                    .load::<SqlOccurrence>(&*self.0)
                    .expect("Loading from database failed.")
                    .into_iter()
                    .map(|sql_occurrence| {
                        let (_, occurrence) = sql_occurrence.into();

                        occurrence
                    })
                    .collect();

                let (id, event) = sql_event.into();
                
                (id, (event, occrs))
            })
            .collect()
    }

    fn create(&self, item: (Event, Vec<Occurrence>)) -> QueryResult<Self::Id> {
        use db::schema::events::dsl::events;
        let sql_event: SqlEvent = item.0.into();
        diesel::insert_into(events)
            .values(&sql_event)
            .execute(&*self.0)?;

        use db::schema::occurrences::dsl::occurrences;
        let sql_occurrences:Vec<SqlOccurrence> = item.1.into_iter().map(|occurrence| occurrence.into()).collect();
        diesel::insert_into(occurrences)
        .values(&sql_occurrences)
        .execute(&*self.0)?;

        Ok(sql_event.id.into())
    }

    fn read(&self, item_id: Self::Id) -> QueryResult<(Event, Vec<Occurrence>)> {
        use db::schema::events::dsl::events;
        events.find(item_id.into()).first::<SqlEvent>(&*self.0).map(|sql_event| {
            let (_, event) = sql_event.into();

            (event, vec![])
        })
    }

    fn update(&self, item_id: Self::Id, new_item: (Event, Vec<Occurrence>)) -> QueryResult<(Event, Vec<Occurrence>)> {
        use db::SqlId;

        let raw_id: SqlId = item_id.into();
        use db::schema::events::dsl::events;
        let (_, previous): (Id, Event) = events.find(&raw_id).first::<SqlEvent>(&*self.0)?.into();

        diesel::update(events.find(&raw_id))
            .set::<SqlEvent>(new_item.0.into())
            .execute(&*self.0)?;

        Ok(previous)
    }

    fn delete(&self, id: Self::Id) -> QueryResult<(Event, Vec<Occurrence>)> {
        use db::SqlId;

        let raw_id: SqlId = id.into();
        use db::schema::events::dsl::events;
        let (_, previous): (Id, Event) = events.find(&raw_id).first::<SqlEvent>(&*self.0)?.into();

        diesel::delete(events.find(&raw_id)).execute(&*self.0)?;

        Ok(previous)
    }
}

pub struct StoreFairing;

impl Fairing for StoreFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Events Store Fairing",
            kind: fairing::Kind::Attach,
        }
    }

    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let result = db::Connection::fairing()
            .on_attach(rocket)
            .and_then(|rocket| db::initialize(rocket));

        result
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Store {
    type Error = <db::Connection as FromRequest<'a, 'r>>::Error;

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        db::Connection::from_request(request).map(Store)
    }
}
