mod db;
#[macro_use]
pub mod action;
pub mod routes;

use std::collections::HashMap;

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

#[derive(Deserialize, Serialize, Debug)]
pub struct Overview {
    pub locations: HashMap<Id, Location>,
    pub events: HashMap<Id, EventWithOccurrences>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EventWithOccurrences {
    pub event: Event,
    pub occurrences: Vec<Occurrence>,
}

pub type Id = Uuid;

pub struct Store(db::Connection);

impl Store {
    pub fn fairing() -> StoreFairing {
        StoreFairing
    }

    pub fn read_all(&self) -> Overview {
        let locs: HashMap<Id, Location> = self.all();
        let evts: HashMap<Id, EventWithOccurrences> = self.all();

        Overview {
            locations: locs,
            events: evts,
        }
    }
}

#[derive(Debug)]
pub enum DeleteError<T> {
    DieselError(diesel::result::Error),
    Dependency(Vec<T>),
}

impl Actions<EventWithOccurrences> for Store {
    type Id = Id;
    type DeleteError = DeleteError<Occurrence>;

    fn all(&self) -> HashMap<Self::Id, EventWithOccurrences> {
        use db::schema::events::dsl::events;

        events
            .load::<SqlEvent>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_event| {
                let occurrences: Vec<Occurrence> = SqlOccurrence::belonging_to(&sql_event)
                    .load::<SqlOccurrence>(&*self.0)
                    .expect("Loading from database failed.")
                    .into_iter()
                    .map(|sql_occurrence| {
                        let (_, occurrence) = sql_occurrence.into();

                        occurrence
                    })
                    .collect();

                let (id, event) = sql_event.into();

                (id, EventWithOccurrences { event, occurrences })
            })
            .collect()
    }

    fn create(&self, item: EventWithOccurrences) -> QueryResult<Self::Id> {
        use db::schema::events::dsl::events;
        let sql_event: SqlEvent = item.event.into();
        diesel::insert_into(events)
            .values(&sql_event)
            .execute(&*self.0)?;

        use db::schema::occurrences::dsl::occurrences;
        let sql_occurrences: Vec<SqlOccurrence> = item
            .occurrences
            .into_iter()
            .map(|occurrence| (occurrence, sql_event.id.clone()).into())
            .collect();
        diesel::insert_into(occurrences)
            .values(&sql_occurrences)
            .execute(&*self.0)?;

        Ok(sql_event.id.into())
    }

    fn read(&self, item_id: Self::Id) -> QueryResult<EventWithOccurrences> {
        use db::schema::events::dsl::events;
        use db::SqlId;
        let sql_event = events
            .find(SqlId::from(item_id))
            .first::<SqlEvent>(&*self.0)?;

        let occurrences: Vec<Occurrence> = SqlOccurrence::belonging_to(&sql_event)
            .load::<SqlOccurrence>(&*self.0)?
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        let (_, event) = sql_event.into();

        Ok(EventWithOccurrences { event, occurrences })
    }

    fn update(
        &self,
        item_id: Self::Id,
        new_item: EventWithOccurrences,
    ) -> QueryResult<EventWithOccurrences> {
        use db::SqlId;

        let raw_id: SqlId = item_id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events.find(raw_id.clone()).first::<SqlEvent>(&*self.0)?;

        let associated_occurrences = SqlOccurrence::belonging_to(&sql_previous);
        let previous_occurrences: Vec<Occurrence> = associated_occurrences
            .load::<SqlOccurrence>(&*self.0)?
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        diesel::delete(associated_occurrences).execute(&*self.0)?;

        let new_sql_item: SqlEvent = new_item.event.into();
        diesel::update(&sql_previous)
            .set(new_sql_item)
            .execute(&*self.0)?;

        use db::schema::occurrences::dsl::occurrences as occurrences_table;
        let sql_occurrences: Vec<SqlOccurrence> = new_item
            .occurrences
            .into_iter()
            .map(|occurrence| (occurrence, raw_id.clone()).into())
            .collect();
        print!("{:?}", sql_occurrences);
        diesel::insert_into(occurrences_table)
            .values(&sql_occurrences)
            .execute(&*self.0)?;

        let (_, previous) = sql_previous.into();
        Ok(EventWithOccurrences {
            event: previous,
            occurrences: previous_occurrences,
        })
    }

    fn delete(&self, id: Self::Id) -> Result<EventWithOccurrences, Self::DeleteError> {
        use db::SqlId;

        let raw_id: SqlId = id.into();
        use db::schema::events::dsl::events;
        let sql_previous = events
            .find(raw_id)
            .first::<SqlEvent>(&*self.0)
            .map_err(DeleteError::DieselError)?;

        let occurrences: Vec<Occurrence> = SqlOccurrence::belonging_to(&sql_previous)
            .load::<SqlOccurrence>(&*self.0)
            .expect("Loading from database failed.")
            .into_iter()
            .map(|sql_occurrence| {
                let (_, occurrence) = sql_occurrence.into();

                occurrence
            })
            .collect();

        if occurrences.len() > 0 {
            return Err(DeleteError::Dependency(occurrences));
        }

        diesel::delete(&sql_previous)
            .execute(&*self.0)
            .map_err(DeleteError::DieselError)?;

        let (_, previous) = sql_previous.into();
        Ok(EventWithOccurrences {
            event: previous,
            occurrences,
        })
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
